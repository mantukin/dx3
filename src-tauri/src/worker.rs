use hidapi::HidApi;
use vigem_client::{Client, XGamepad, TargetId, Xbox360Wired};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashSet;
use log::{info, warn};
use tauri::Manager; // For emit_all

use crate::state::SharedState;
use crate::mapping::{GamepadState, parse_dualsense, parse_ds4, MappingTarget};
use crate::hidhide;
use crate::dualsense::{send_dualsense_output, send_raw_output};
use crate::crc;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, KEYBDINPUT, MOUSEINPUT, KEYBD_EVENT_FLAGS,
    VIRTUAL_KEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_EXTENDEDKEY,
    MapVirtualKeyW, MAPVK_VK_TO_VSC,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, 
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, 
    MOUSEEVENTF_MOVE, MOUSEEVENTF_WHEEL,
    INPUT_KEYBOARD, INPUT_MOUSE
};

const VID_SONY: u16 = 0x054C;
const PID_DS4_V1: u16 = 0x05C4;
const PID_DS4_V2: u16 = 0x09CC;
const PID_DUALSENSE: u16 = 0x0CE6;

// --- Background Controller Thread ---

pub fn controller_thread(state: Arc<Mutex<SharedState>>, app_handle: tauri::AppHandle) {
    // Helper to update status safely
    let set_status = |s: &str, dev: &str| {
        let mut locked = state.lock().unwrap();
        locked.status = s.to_string();
        locked.device_name = dev.to_string();
        // Clear visuals if we are not actively connected
        if s.contains("Wait") || s.contains("Disconnected") || s.contains("Searching") {
            locked.gamepad = GamepadState::default();
        }
    };

    let mut last_sent_state = GamepadState::default();
    let mut consecutive_simple_reconnects = 0;

    // Outer Loop: Handles Driver/HID Initialization Retries
    loop {
        // Check for app exit request
        if state.lock().unwrap().should_exit {
            break;
        }

        set_status("Initializing ViGEm...", "None");
        
        // Connect to ViGEmBus
        let vigem = match Client::connect() {
            Ok(c) => {
                {
                    let mut s = state.lock().unwrap();
                    s.vigembus_available = true;
                }
                let _ = app_handle.emit_all("update-state", &*state.lock().unwrap());
                c
            },
            Err(e) => {
                {
                    let mut s = state.lock().unwrap();
                    s.vigembus_available = false;
                }
                let err_msg = format!("ViGEmBus Error: {}", e);
                set_status(&err_msg, "None");
                let _ = app_handle.emit_all("update-state", &*state.lock().unwrap());
                
                // Manual Retry Loop
                // Wait 2s before retrying. User can click 'Check' to set should_reinit, 
                // which will be caught at the start of the next outer loop iteration.
                thread::sleep(Duration::from_secs(2));
                
                let mut s = state.lock().unwrap();
                if s.should_exit { return; }
                s.should_reinit = false; // Clear any pending reinit to start fresh
                continue;
            }
        };
        
        // Attempt to whitelist self in HidHide
        let hh_installed = hidhide::is_installed();
        {
            let mut s = state.lock().unwrap();
            s.hidhide_available = hh_installed;
        }
        let _ = app_handle.emit_all("update-state", &*state.lock().unwrap());

        if hh_installed {
            if let Err(e) = hidhide::whitelist_self() {
                warn!("Failed to whitelist self in HidHide: {}", e);
            }
            // Give Windows a moment to apply HidHide whitelist before opening HID
            thread::sleep(Duration::from_millis(500));
        }

        set_status("Scanning for controllers...", "None");
        
        let mut hid = match HidApi::new() {
            Ok(h) => h,
            Err(e) => {
                let err_msg = format!("HID Error: {}", e);
                set_status(&err_msg, "None");
                let _ = app_handle.emit_all("update-state", &*state.lock().unwrap());
                
                thread::sleep(Duration::from_secs(2));
                let mut s = state.lock().unwrap();
                if s.should_exit { return; }
                s.should_reinit = false;
                continue;
            }
        };

        let mut no_device_counter = 0;

        // Main scanning loop
        loop {
            // Check pause state
            let paused = state.lock().unwrap().is_paused;
            if paused {
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            
            if state.lock().unwrap().should_exit {
                break;
            }

            // Check for Manual Driver Refresh Request
            {
                let mut locked = state.lock().unwrap();
                if locked.should_reinit {
                    locked.should_reinit = false;
                    info!("Manual driver refresh requested. Re-initializing subsystems...");
                    break; 
                }
            }

            // Refresh device list to detect hotplugged controllers
            if let Err(e) = hid.refresh_devices() {
                warn!("Failed to refresh HID devices: {}", e);
                break;
            }

            // ... (scanning logic) ...
            // (I will keep the rest of the scanning logic and just update the end of loop)
            // ...


            let devices = hid.device_list();
            let mut found = false;
            let mut log_buf = String::new();
            let mut best_candidate = None;

            for device_info in devices {
            if device_info.vendor_id() == VID_SONY {
                let pid = device_info.product_id();
                let iface = device_info.interface_number();
                let up = device_info.usage_page();
                let u = device_info.usage();
                
                log_buf.push_str(&format!("PID:{:04X} Iface:{} UP:{} U:{} \nPath:{}\n\n", 
                    pid, iface, up, u, device_info.path().to_str().unwrap_or("?")));

                let is_ds4 = pid == PID_DS4_V1 || pid == PID_DS4_V2;
                let is_dualsense = pid == PID_DUALSENSE;

                if is_ds4 || is_dualsense {
                    // Score candidates
                    // Priority 1: Generic Desktop (1) + Gamepad (5)
                    if up == 1 && u == 5 {
                        best_candidate = Some(device_info);
                        break; // Found perfect match
                    }
                    // Priority 2: If no UP/U available (0), assume it might be it (fallback)
                    if best_candidate.is_none() && up == 0 {
                        best_candidate = Some(device_info);
                    }
                }
            }
        }

        if let Some(device_info) = best_candidate {
            {
                let name = device_info.product_string().unwrap_or("Unknown").to_string();
                let dev_path_clone = device_info.path().to_str().unwrap_or("?").to_string();
                let pid = device_info.product_id();
                let is_dualsense = pid == PID_DUALSENSE;
                
                // Identify Instance ID for HidHide EARLY (Pre-emptive Strike)
                let instance_id = hidhide::path_to_instance_id(device_info.path().to_str().unwrap_or(""));
                let mut is_hidden = false;

                // Attempt to hide BEFORE opening the device to race against Steam/Games
                if let Some(inst) = &instance_id {
                    let mut s = state.lock().unwrap();
                    if s.hide_controller {
                        if let Ok(_) = hidhide::hide_device(inst) {
                            s.hidden_device_id = Some(inst.clone());
                            is_hidden = true;
                        }
                    }
                }

                if let Ok(device) = device_info.open_device(&hid) {
                    set_status(&format!("Active (Iface {})", device_info.interface_number()), &name);
                    state.lock().unwrap().device_path_str = dev_path_clone;
                    state.lock().unwrap().detected_devices_log = log_buf.clone();
                    found = true;

                    // Create Virtual Xbox 360 (but don't plugin yet)
                    let mut target = Xbox360Wired::new(vigem.try_clone().unwrap(), TargetId::XBOX360_WIRED);
                    let mut is_plugged = false;
                    
                    // DualSense Connection Mode
                    let is_bt = is_dualsense && device_info.interface_number() == -1;

                    // === CRITICAL: Enable Enhanced Mode for Bluetooth ===
                    // DualSense defaults to Simple Mode (DirectInput) over BT,
                    // where LED/Haptics/Triggers are unavailable. Reading Feature Report 0x09
                    // (serial number) or 0x20 (firmware) activates Enhanced Mode.
                    if is_dualsense && is_bt {
                        let mut feature_buf = [0u8; 64];
                        feature_buf[0] = 0x09; // Feature Report ID for serial number
                        match device.get_feature_report(&mut feature_buf) {
                            Ok(len) => {
                                info!("DualSense BT: Enhanced Mode activated via Feature Report 0x09 ({} bytes)", len);
                            }
                            Err(e) => {
                                warn!("DualSense BT: Failed to read Feature Report 0x09: {} — LED may not work!", e);
                                // Try alternative Feature Report 0x20
                                feature_buf[0] = 0x20;
                                if let Ok(len) = device.get_feature_report(&mut feature_buf) {
                                    info!("DualSense BT: Enhanced Mode activated via Feature Report 0x20 ({} bytes)", len);
                                }
                            }
                        }
                    }

                    // Initial LED Setup
                    if is_dualsense {
                        let (r, g, b, bright, show_bat, l2_m, l2_s, l2_f, r2_m, r2_s, r2_f, pled_bright) = {
                            let s = state.lock().unwrap();
                            (s.rgb_r, s.rgb_g, s.rgb_b, s.rgb_brightness, s.show_battery_led,
                             s.trigger_l2_mode, s.trigger_l2_start, s.trigger_l2_force,
                             s.trigger_r2_mode, s.trigger_r2_start, s.trigger_r2_force,
                             s.player_led_brightness)
                        };
                        let pled = if show_bat {
                            get_battery_led_mask(last_sent_state.battery)
                        } else {
                            0x04 // Standard Center LED
                        };

                        // Apply brightness scaling
                        let bf = bright as f32 / 255.0;
                        let fr = (r as f32 * bf) as u8;
                        let fg = (g as f32 * bf) as u8;
                        let fb = (b as f32 * bf) as u8;
                        
                        // Wake-up to initialize controller LEDs (+ short rumble)
                        if is_bt {
                            crate::dualsense::send_led_init(&device, 0, pled, fr, fg, fb);
                        } else {
                            crate::dualsense::send_led_init_usb(&device, pled, fr, fg, fb);
                        }
                        thread::sleep(Duration::from_millis(50));
                        
                        send_dualsense_output(&device, is_bt, fr, fg, fb, pled, pled_bright, 0, l2_m, l2_s, l2_f, r2_m, r2_s, r2_f);
                    }

                    // Input Loop State
                    let mut simple_mode_counter = 0;
                    let mut buf = [0u8; 128];
                    let mut last_led_update = Instant::now();
                    let mut last_sweep_update = Instant::now();
                    let mut last_fuzzer_update = Instant::now();
                    let mut last_periodic_update = Instant::now();
                    let mut last_hidhide_check = Instant::now();
                    let mut last_ui_update = Instant::now();
                    let mut last_pad_update = Instant::now();
                    
                    let mut active_keys = HashSet::new();
                    let mut active_mouse = HashSet::new();
                    let mut mouse_acc = (0.0f32, 0.0f32);
                    let mut scroll_acc = 0.0f32;
                    let mut smoothed_axes = [0.0f32; 4]; // [LX, LY, RX, RY]
                    
                    // Touchpad State
                    let mut last_touch_x = 0u16;
                    let mut last_touch_y = 0u16;
                    let mut last_touch_active = false;
                    let mut smoothed_touch = (0.0f32, 0.0f32); // [dx, dy]

                    let mut local_mappings = {
                        let mut s = state.lock().unwrap();
                        s.mappings_changed = false; 
                        s.mappings.clone()
                    };
                    let (mut local_deadzone_l, mut local_deadzone_r, mut local_mouse_sens_l, mut local_mouse_sens_r, mut local_mouse_sens_touchpad) = {
                        let s = state.lock().unwrap();
                        (s.deadzone_left, s.deadzone_right, s.mouse_sens_left, s.mouse_sens_right, s.mouse_sens_touchpad)
                    };
                    
                        let mut last_report_buf = [0u8; 80];
                        let mut last_report_len = 0;
                        
                        // State tracking for UI optimization (Deduplication)
                        let mut last_emitted_gamepad = GamepadState::default();
                        let mut last_emitted_status = String::new();
                        let mut last_emit_time = Instant::now();
                    
                        // Burst Loop
                        loop {                        // 1. Sync Mappings and settings
                        let should_thread_exit = {
                            let mut s = state.lock().unwrap();
                            if s.should_exit {
                                info!("Shutdown signal received. Resetting controller LEDs...");
                                if is_dualsense {
                                    // Reset to standard Blue (0, 0, 255) and Center LED (0x04)
                                    // We also disable adaptive triggers (0)
                                    send_dualsense_output(
                                        &device, is_bt, 
                                        0, 0, 255, 0x04, s.player_led_brightness, s.bt_sequence,
                                        0, 0, 0, 0, 0, 0
                                    );
                                }
                                true
                            } else {
                                if s.mappings_changed {
                                    local_mappings = s.mappings.clone();
                                    s.mappings_changed = false;
                                }
                                local_deadzone_l = s.deadzone_left;
                                local_deadzone_r = s.deadzone_right;
                                local_mouse_sens_l = s.mouse_sens_left;
                                local_mouse_sens_r = s.mouse_sens_right;
                                local_mouse_sens_touchpad = s.mouse_sens_touchpad;
                                false
                            }
                        };

                        if should_thread_exit { return; }

                        // 2. HIDHIDE Check (Rarely)
                        if last_hidhide_check.elapsed().as_secs() >= 1 {
                            if let Some(inst_id) = &instance_id {
                                let mut s = state.lock().unwrap();
                                let want_hide = s.hide_controller;
                                if want_hide && !is_hidden {
                                    if let Ok(_) = hidhide::hide_device(inst_id) {
                                        is_hidden = true;
                                        s.hidden_device_id = Some(inst_id.clone());
                                    }
                                } else if !want_hide && is_hidden {
                                    let _ = hidhide::unhide_device(inst_id);
                                    is_hidden = false;
                                    s.hidden_device_id = None;
                                }
                            }
                            last_hidhide_check = Instant::now();
                        }

                        // 3. Read Packet (Burst Mode)
                        // Read with timeout 10ms to allow housekeeping when idle
                        match device.read_timeout(&mut buf, 10) {
                            Ok(0) => {
                                // Timeout - Controller Idle or slow connection
                                // We call update_virtual_pad with last_sent_state to keep mouse moving smoothly
                                let dt = last_pad_update.elapsed().as_secs_f32();
                                last_pad_update = Instant::now();
                                update_virtual_pad(&mut target, &last_sent_state, &local_mappings, &mut active_keys, &mut active_mouse, &mut mouse_acc, &mut scroll_acc, false, local_deadzone_l, local_deadzone_r, &mut smoothed_axes, local_mouse_sens_l, local_mouse_sens_r, local_mouse_sens_touchpad, &mut last_touch_x, &mut last_touch_y, &mut last_touch_active, &mut smoothed_touch, dt);
                            },
                            Ok(size) => {
                                // Process Packet
                                let report = &buf[0..size];
                                let parsed_state = if is_dualsense {
                                    parse_dualsense(report, is_bt)
                                } else {
                                    parse_ds4(report)
                                };

                                if let Some(s) = parsed_state {
                                    // Connection Mode Detection Logic (Tolerant to initial Simple Mode bursts)
                                    let report_id = report[0];
                                    
                                    if is_dualsense && is_bt {
                                        let mut locked = state.lock().unwrap();
                                        
                                        if locked.connection_mode != "Native (BT 0x31)" {
                                            if report_id == 0x31 {
                                                // SUCCESS: Native mode confirmed
                                                locked.connection_mode = "Native (BT 0x31)".to_string();
                                                consecutive_simple_reconnects = 0;
                                                simple_mode_counter = 0;
                                            } else if report_id == 0x01 {
                                                // WARNING: Simple mode detected
                                                simple_mode_counter += 1;
                                                
                                                if locked.connection_mode.is_empty() {
                                                     locked.connection_mode = format!("Waiting... ({})", simple_mode_counter);
                                                }

                                                // If we receive > 200 packets (approx 0.5 - 1s) of 0x01 without 0x31, THEN we try to fix it.
                                                if simple_mode_counter > 200 {
                                                    if consecutive_simple_reconnects < 1 {
                                                        warn!("DualSense stuck in Simple Mode (>200 pkts). Auto-reconnecting... (Attempt {})", consecutive_simple_reconnects + 1);
                                                        locked.should_disconnect = true;
                                                        consecutive_simple_reconnects += 1;
                                                        locked.connection_mode = "Simple (Stuck) - RECONNECTING...".to_string();
                                                    } else {
                                                        // We already tried reconnecting once and it didn't help. 
                                                        // Stop spamming reconnects and just accept fate.
                                                        if simple_mode_counter == 201 { // Log once
                                                            warn!("DualSense stuck in Simple Mode after reconnect. Giving up.");
                                                            locked.connection_mode = "Simple (BT 0x01) - FAILED TO FIX".to_string();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        // USB or DS4 - Instant detection is fine
                                        let mut locked = state.lock().unwrap();
                                        if locked.connection_mode.is_empty() {
                                            let mode = if is_dualsense {
                                                "Native (USB 0x01)".to_string()
                                            } else {
                                                format!("DS4 (0x{:02X})", report_id)
                                            };
                                            locked.connection_mode = mode;
                                        }
                                    }

                                    // Plugin Virtual Pad if needed
                                    if !is_plugged {
                                        if let Err(e) = target.plugin() {
                                            set_status(&format!("ViGEm Error: {}", e), &name);
                                            break; 
                                        }
                                        let _ = target.wait_ready();
                                        is_plugged = true;
                                        info!("Virtual Xbox 360 plugged in and ready.");
                                        set_status("Virtual Pad: Ready", &name);
                                    }

                                    // Update Virtual Pad (Always for smooth mouse, but pass change flag for ViGEm)
                                    let changed = s != last_sent_state;
                                    let dt = last_pad_update.elapsed().as_secs_f32();
                                    last_pad_update = Instant::now();
                                    update_virtual_pad(&mut target, &s, &local_mappings, &mut active_keys, &mut active_mouse, &mut mouse_acc, &mut scroll_acc, changed, local_deadzone_l, local_deadzone_r, &mut smoothed_axes, local_mouse_sens_l, local_mouse_sens_r, local_mouse_sens_touchpad, &mut last_touch_x, &mut last_touch_y, &mut last_touch_active, &mut smoothed_touch, dt);
                                    last_sent_state = s;

                                    // Batch this packet
                                    last_report_len = size.min(80);
                                    last_report_buf[..last_report_len].copy_from_slice(&report[..last_report_len]);
                                }
                                
                                // DRAIN QUEUE: Check if more data is available immediately
                                // This prevents building up latency if input > processing speed
                                // We loop here up to 10 times to drain buffer
                                for _ in 0..10 {
                                    // Non-blocking read (timeout 0)
                                    match device.read_timeout(&mut buf, 0) {
                                        Ok(sz) if sz > 0 => {
                                             // Process this packet too!
                                             let sub_report = &buf[0..sz];
                                             let sub_parsed = if is_dualsense {
                                                 parse_dualsense(sub_report, is_bt)
                                             } else {
                                                 parse_ds4(sub_report)
                                             };
                                             
                                             if let Some(sub_s) = sub_parsed {
                                                 // Update Virtual Pad immediately for smooth motion
                                                 let changed = sub_s != last_sent_state;
                                                 let dt = last_pad_update.elapsed().as_secs_f32();
                                                 last_pad_update = Instant::now();
                                                 update_virtual_pad(&mut target, &sub_s, &local_mappings, &mut active_keys, &mut active_mouse, &mut mouse_acc, &mut scroll_acc, changed, local_deadzone_l, local_deadzone_r, &mut smoothed_axes, local_mouse_sens_l, local_mouse_sens_r, local_mouse_sens_touchpad, &mut last_touch_x, &mut last_touch_y, &mut last_touch_active, &mut smoothed_touch, dt);
                                                 last_sent_state = sub_s;
                                                 
                                                 // Batch this packet (overwrite previous)
                                                 last_report_len = sz.min(80);
                                                 last_report_buf[..last_report_len].copy_from_slice(&sub_report[..last_report_len]);
                                             }
                                        }
                                        _ => break, // Queue empty or error
                                    }
                                }
                            }
                            Err(_) => {
                                warn!("Device read error, disconnecting...");
                                break;
                            }
                        }

                        // REMOVED AGGRESSIVE LOCKING HERE

                        // UI Update (Throttled & Deduplicated) 
                        // Reduce max rate to 30 FPS (32ms) to save JS GC pressure
                        if last_ui_update.elapsed().as_millis() >= 32 {
                            let mut locked = state.lock().unwrap();
                            let should_emit = locked.ui_visible;
                            
                            if should_emit {
                                locked.gamepad = last_sent_state;
                                locked.virtual_pad_active = is_plugged;
                                
                                locked.gamepad.left_x = smoothed_axes[0];
                                locked.gamepad.left_y = smoothed_axes[1];
                                locked.gamepad.right_x = smoothed_axes[2];
                                locked.gamepad.right_y = smoothed_axes[3];

                                locked.last_update = locked.last_update.wrapping_add(1);
                                locked.raw_report[..last_report_len].copy_from_slice(&last_report_buf[..last_report_len]);

                                // OPTIMIZATION: Only emit if state changed visually or it's been >1s (keep-alive)
                                // This prevents flooding JS with identical JSONs, stopping memory leaks.
                                let changed = locked.gamepad != last_emitted_gamepad || 
                                              locked.status != last_emitted_status ||
                                              locked.should_send_leds || 
                                              locked.mappings_changed ||
                                              last_emit_time.elapsed().as_millis() > 1000;

                                if changed {
                                    let mut current_state = locked.clone();
                                    
                                    // Optimization: Clear heavy logs if debug is not active
                                    if !current_state.debug_active {
                                        current_state.detected_devices_log.clear();
                                        current_state.protocol_log.clear();
                                        current_state.last_packet_hex.clear();
                                    }
                                    
                                    // Update tracking vars
                                    last_emitted_gamepad = current_state.gamepad;
                                    last_emitted_status = current_state.status.clone();
                                    last_emit_time = Instant::now();

                                    drop(locked); // Unlock before emitting
                                    let _ = app_handle.emit_all("update-state", &current_state);
                                }
                            }
                            last_ui_update = Instant::now();
                        }

                        // 3. LED / Fuzzer Housekeeping (Throttled 1ms)
                        if last_led_update.elapsed().as_millis() >= 1 {
                             let (active, step, manual_id, manual_flag, manual_rgb, manual_r, manual_g, manual_b, do_manual, seq, crc_mode, disable_period, pp_off, pp_val, do_pp, manual_pled, manual_pb, manual_pb_off, sweep_active, sweep_timeout, bt_flags, bt_flags2, bt_len, use_feature, do_proto_scan, force_leds, force_triggers, disconnect) = {
                                let mut s = state.lock().unwrap();
                                let send = s.should_send_manual;
                                let send_pp = s.should_send_pinpoint;
                                let scan = s.protocol_scan_active;
                                let f_leds = s.should_send_leds;
                                let f_triggers = s.should_send_triggers;
                                let disc = s.should_disconnect;
                                s.should_send_manual = false; 
                                s.should_send_pinpoint = false;
                                s.should_send_leds = false;
                                s.should_send_triggers = false;
                                s.should_disconnect = false;
                                let sq = s.bt_sequence;
                                s.bt_sequence = s.bt_sequence.wrapping_add(1);
                                (s.fuzzer_active, s.fuzzer_step, s.manual_report_id, s.manual_flag_offset, s.manual_rgb_offset, s.manual_r, s.manual_g, s.manual_b, send, sq, s.crc_seed_idx, s.disable_periodic, s.pinpoint_offset, s.pinpoint_value, send_pp, s.manual_player_led, s.manual_pled_bright, s.manual_pled_bright_off, s.sweep_active, s.sweep_timeout_ms, s.bt_flag_val, s.bt_flag_val2, s.manual_bt_len, s.send_as_feature, scan, f_leds, f_triggers, disc)
                            };

                            if disconnect {
                                info!("Reconnect requested.");
                                {
                                    let mut s = state.lock().unwrap();
                                    s.status = "Reconnecting...".to_string();
                                }
                                
                                if is_dualsense && is_bt {
                                    // Send a series of power off packets
                                    for i in 0..10 {
                                        crate::dualsense::send_power_off(&device, true, seq.wrapping_add(i as u8));
                                        thread::sleep(Duration::from_millis(10));
                                    }
                                }
                                
                                let mut s = state.lock().unwrap();
                                s.connection_mode = String::new();
                                // We do NOT pause here anymore, so it acts as a Reconnect
                                // s.status = "Paused (Manual Disconnect)".to_string();
                                // s.is_paused = true;
                                drop(s);
                                
                                break; // Exits inner loop, triggering re-scan immediately
                            }

                            if do_proto_scan {
                                run_protocol_scan(&device, seq, &state);
                            }

                            // Manual / Pinpoint / Fuzzer / Periodic logic
                            if do_manual {
                                let res = send_raw_output(&device, manual_id, manual_flag, manual_rgb, manual_r, manual_g, manual_b, seq, crc_mode, manual_pled, manual_pb, manual_pb_off, bt_flags, bt_flags2, bt_len, use_feature);
                                
                                let (status, hex) = match res {
                                    Ok((n, hex)) => (format!("OK ({} bytes)", n), hex),
                                    Err(e) => {
                                        if let Some(idx) = e.find("| Hex: ") {
                                            let err_msg = &e[..idx];
                                            let hex_part = &e[idx + 7..];
                                            (format!("Error: {}", err_msg), hex_part.to_string())
                                        } else {
                                            (format!("Error: {}", e), String::new())
                                        }
                                    }
                                };

                                let mut s = state.lock().unwrap();
                                s.last_write_status = status;
                                s.last_packet_hex = hex;
                            }

                            if do_pp {
                                // Pinpoint Logic
                                let mut report = [0u8; 78];
                                let rep_id = if is_bt { 0x31 } else { 0x02 };
                                report[0] = rep_id;
                                if is_bt {
                                    report[1] = (seq << 4) | 0x02; 
                                    report[2] = 0xF7; // Main Flags
                                    report[3] = 0x15; // LED Flags
                                    report[4] = 0x00; // No rumble
                                } else { 
                                    report[2] = 0xF7; 
                                }
                                if pp_off < 78 { report[pp_off] = pp_val; }
                                if is_bt {
                                    let checksum = crc::crc32_bt(&report[0..74]);
                                    report[74] = (checksum & 0xFF) as u8;
                                    report[75] = ((checksum >> 8) & 0xFF) as u8;
                                    report[76] = ((checksum >> 16) & 0xFF) as u8;
                                    report[77] = ((checksum >> 24) & 0xFF) as u8;
                                }
                                let res = if is_bt { device.write(&report) } else { device.write(&report[0..64]) };
                                let status = match res { Ok(_) => format!("PP OK ({} -> [{}])", pp_val, pp_off), Err(e) => format!("Error: {}", e) };
                                state.lock().unwrap().last_write_status = status;
                            }
                            
                            if sweep_active {
                                if last_sweep_update.elapsed().as_millis() >= sweep_timeout as u128 {
                                    run_sweep_logic(&device, step, seq, &state, sweep_timeout);
                                    last_sweep_update = Instant::now();
                                }
                            } else if active {
                                if last_fuzzer_update.elapsed().as_millis() >= 50 {
                                    run_fuzzer_logic(&device, step, seq, crc_mode, bt_flags, bt_len, use_feature, &state);
                                    last_fuzzer_update = Instant::now();
                                }
                            } else {
                                // Periodic Battery/LED update
                                // SAFETY: Do NOT send 0x31 output reports while the controller is still in Simple Mode (0x01).
                                // This prevents "fighting" the firmware and causing the red LED glitch.
                                let safe_to_send = simple_mode_counter == 0;
                                
                                if safe_to_send && (force_leds || force_triggers || (!disable_period && last_periodic_update.elapsed().as_millis() >= 1000)) {
                                    let (r, g, b, bright, show_bat, l2_m, l2_s, l2_f, r2_m, r2_s, r2_f, pled_bright) = {
                                        let s = state.lock().unwrap();
                                        (s.rgb_r, s.rgb_g, s.rgb_b, s.rgb_brightness, s.show_battery_led,
                                         s.trigger_l2_mode, s.trigger_l2_start, s.trigger_l2_force,
                                         s.trigger_r2_mode, s.trigger_r2_start, s.trigger_r2_force,
                                         s.player_led_brightness)
                                    };
                                    
                                    let pled = if show_bat {
                                        get_battery_led_mask(last_sent_state.battery)
                                    } else {
                                        0x04 // Standard Center LED
                                    };

                                    // Apply brightness scaling
                                    let bf = bright as f32 / 255.0;
                                    let fr = (r as f32 * bf) as u8;
                                    let fg = (g as f32 * bf) as u8;
                                    let fb = (b as f32 * bf) as u8;

                                    send_dualsense_output(&device, is_bt, fr, fg, fb, pled, pled_bright, seq, l2_m, l2_s, l2_f, r2_m, r2_s, r2_f);
                                    last_periodic_update = Instant::now();
                                }
                            }

                            // Force UI update after LED/Fuzzer actions to show status immediately
                            // But only if visible!
                            let locked = state.lock().unwrap();
                            if locked.ui_visible {
                                let _ = app_handle.emit_all("update-state", &*locked);
                            }
                            last_led_update = Instant::now();
                        }
                    }
                    
                    // Unplug if loop breaks
                    if is_plugged {
                        update_virtual_pad(&mut target, &GamepadState::default(), &[], &mut active_keys, &mut active_mouse, &mut mouse_acc, &mut scroll_acc, true, local_deadzone_l, local_deadzone_r, &mut [0.0f32; 4], local_mouse_sens_l, local_mouse_sens_r, 0.0, &mut 0, &mut 0, &mut false, &mut (0.0, 0.0), 0.0);
                        let _ = target.unplug();
                    }
                    if is_hidden {
                        if let Some(inst_id) = &instance_id {
                            let _ = hidhide::unhide_device(inst_id);
                            state.lock().unwrap().hidden_device_id = None;
                        }
                    }
                    set_status("Disconnected", "None");
                    {
                        let mut locked = state.lock().unwrap();
                        locked.virtual_pad_active = false;
                        locked.connection_mode = String::new();
                    }
                    let _ = app_handle.emit_all("update-state", &*state.lock().unwrap());
                    
                    // Pause to allow physical controller disconnection
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }

        if !found {
            // SOFT REINIT: If no device found for 5 iterations (~10s), 
            // break to outer loop to refresh HID and whitelist.
            no_device_counter += 1;
            if no_device_counter > 5 {
                warn!("No device found for 10s. Refreshing HID subsystems...");
                break; 
            }

            state.lock().unwrap().detected_devices_log = log_buf;
            set_status("Searching for controller...", "None");
            let _ = app_handle.emit_all("update-state", &*state.lock().unwrap());
            thread::sleep(Duration::from_secs(2));
        } else {
            no_device_counter = 0;
        }
    }
}
}



// Helper for Fuzzer/Sweep to keep main loop clean
fn run_sweep_logic(device: &hidapi::HidDevice, current_step: usize, seq: u8, state: &Arc<Mutex<SharedState>>, _sweep_timeout: u64) {
    let mut report_bt = [0u8; 78];
    report_bt[0] = 0x31;
    report_bt[1] = (seq << 4) | 0x02; 
    report_bt[2] = 0x15;
    let log_msg;

    if current_step < 80 {
        report_bt[2] = 0xF7; report_bt[3] = 0x15; report_bt[4] = 0x00;
        if current_step < 75 {
            report_bt[current_step] = 255;
            report_bt[current_step + 1] = 255;
            report_bt[current_step + 2] = 255;
        }
        log_msg = format!("ULTIMATE: Offset Sweep @ {}", current_step);
    } else {
        let flag_phase_step = current_step - 80;
        let flag_byte_idx = (flag_phase_step / 256) + 1; 
        let flag_value = (flag_phase_step % 256) as u8;

        if flag_byte_idx > 5 {
            let mut s = state.lock().unwrap();
            s.fuzzer_step = 0;
            log_msg = "ULTIMATE: Resetting...".to_string();
        } else {
            for i in 44..50 { report_bt[i] = 255; }
            report_bt[flag_byte_idx] = flag_value;
            log_msg = format!("ULTIMATE: Flag[{}] = 0x{:02X} (RGB Fixed)", flag_byte_idx, flag_value);
        }
    }
    
    let mut s = state.lock().unwrap();
    s.fuzzer_log = log_msg.clone();
    s.fuzzer_step += 1;
    if s.fuzzer_step > 2000 { s.fuzzer_step = 0; }
    drop(s);

    let checksum = crc::crc32_bt(&report_bt[0..74]); 
    report_bt[74] = (checksum & 0xFF) as u8;
    report_bt[75] = ((checksum >> 8) & 0xFF) as u8;
    report_bt[76] = ((checksum >> 16) & 0xFF) as u8;
    report_bt[77] = ((checksum >> 24) & 0xFF) as u8;
    let _ = device.write(&report_bt);
    
    let hex_str = report_bt.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ");
    
    let mut s = state.lock().unwrap();
    s.last_write_status = log_msg;
    s.last_packet_hex = hex_str;
}

fn run_fuzzer_logic(device: &hidapi::HidDevice, step: usize, seq: u8, crc_mode: u8, bt_flags: u8, bt_len: usize, use_feature: bool, state: &Arc<Mutex<SharedState>>) {
    let report_id = if step < 10 { 0x02 } else { 0x31 };
    let (flag_off, rgb_off, desc) = if step < 10 {
        (step, 44 + step, format!("USB (0x02) | Flags @ {} | RGB @ {}", step, 44+step))
    } else if step < 30 {
        (step - 10, 45, format!("BT (0x31) | Flags @ {} | RGB @ {}", step-10, 45))
    } else if step < 60 {
        (3, 35 + (step - 30), format!("BT (0x31) | Flags @ 3 | RGB @ {}", 35 + (step - 30)))
    } else {
        let mut s = state.lock().unwrap();
        s.fuzzer_step = 0;
        (0, 0, "Reset".to_string())
    };

    if desc != "Reset" {
        {
            let mut s = state.lock().unwrap();
            s.fuzzer_log = format!("Step {}: {}", step, desc);
            s.fuzzer_step += 1;
        }

        let mut last_res = String::new();
        let mut last_hex = String::new();
        // Burst
        for i in 0..3 {
            let res = send_raw_output(device, report_id, flag_off, rgb_off, 255, 0, 0, seq.wrapping_add(i as u8), crc_mode, 0x04, 0, 0, bt_flags, 0x15, bt_len, use_feature);
            
            let (status, hex) = match res {
                Ok((n, h)) => (format!("OK ({} bytes)", n), h),
                Err(e) => {
                    if let Some(idx) = e.find("| Hex: ") {
                        (format!("Error: {}", &e[..idx]), e[idx + 7..].to_string())
                    } else {
                        (format!("Error: {}", e), String::new())
                    }
                }
            };
            last_res = status;
            last_hex = hex;
            
            thread::sleep(Duration::from_millis(5));
        }
        let mut s = state.lock().unwrap();
        s.last_write_status = last_res;
        s.last_packet_hex = last_hex;
    }
}

fn run_protocol_scan(device: &hidapi::HidDevice, seq: u8, state: &Arc<Mutex<SharedState>>) {
    let mut log = String::from("--- PROTOCOL SCAN START ---\n");
    // 1. Output 0x31
    log.push_str(">> Report 0x31 (Output) Length Scan:\n");
    for l in 60..=80 {
        let res = send_raw_output(device, 0x31, 2, 45, 255, 0, 0, seq, 0, 0, 0, 0, 0xF7, 0x15, l, false);
        log.push_str(&format!("Len {}: {}\n", l, match res { Ok(_) => "OK".to_string(), Err(e) => e }));
        thread::sleep(Duration::from_millis(10));
    }
    // 2. Feature 0x31
    log.push_str("\n>> Report 0x31 (Feature) Length Scan:\n");
    for l in 60..=80 {
        let res = send_raw_output(device, 0x31, 2, 45, 255, 0, 0, seq, 0, 0, 0, 0, 0xF7, 0x15, l, true);
        log.push_str(&format!("Len {}: {}\n", l, match res { Ok(_) => "OK".to_string(), Err(e) => e }));
        thread::sleep(Duration::from_millis(10));
    }
    // 2.5 DS4
    log.push_str("\n>> Report 0x11 (DS4 Output):\n");
    let res_11 = send_raw_output(device, 0x11, 2, 45, 255, 0, 0, seq, 0, 0, 0, 0, 0xF7, 0x15, 78, false);
    log.push_str(&format!("ID 11: {}\n", match res_11 { Ok(_) => "OK".to_string(), Err(e) => e }));

    log.push_str("--- END ---\n");
    let mut s = state.lock().unwrap();
    s.protocol_log = log;
    s.protocol_scan_active = false;
}

fn apply_deadzone(x: f32, y: f32, deadzone: f32) -> (f32, f32) {
    let magnitude = (x * x + y * y).sqrt();
    if magnitude < deadzone {
        (0.0, 0.0)
    } else {
        // Rescale magnitude to start from 0 at the edge of the deadzone
        let rescaled_magnitude = (magnitude - deadzone) / (1.0 - deadzone);
        let ratio = rescaled_magnitude / magnitude;
        (x * ratio, y * ratio)
    }
}

fn get_battery_led_mask(battery: u8) -> u8 {
    // DualSense Player LEDs sequential filling (left to right):
    // 0x01 - 1 LED
    // 0x03 - 2 LEDs
    // 0x07 - 3 LEDs
    // 0x0F - 4 LEDs
    // 0x1F - 5 LEDs
    if battery >= 90 { 0x1F }
    else if battery >= 70 { 0x0F }
    else if battery >= 50 { 0x07 }
    else if battery >= 30 { 0x03 }
    else if battery >= 10 { 0x01 }
    else { 0x00 }
}

fn update_virtual_pad(
    target: &mut Xbox360Wired<Client>, 
    s: &GamepadState, 
    mappings: &[crate::mapping::ButtonMapping], 
    active_keys: &mut HashSet<u16>, 
    active_mouse: &mut HashSet<u8>,
    mouse_acc: &mut (f32, f32),
    scroll_acc: &mut f32,
    state_changed: bool,
    deadzone_l: f32,
    deadzone_r: f32,
    smoothed_axes: &mut [f32; 4],
    sens_l: f32,
    sens_r: f32,
    sens_touchpad: f32,
    last_touch_x: &mut u16,
    last_touch_y: &mut u16,
    last_touch_active: &mut bool,
    smoothed_touch: &mut (f32, f32),
    dt: f32
) {
    let mut gamepad = XGamepad::default();
    let mut raw_buttons: u16 = 0;
    
    let mut current_keys = HashSet::new();
    let mut current_mouse = HashSet::new();
    
    let mut mouse_dx = 0.0f32;
    let mut mouse_dy = 0.0f32;
    let mut scroll_dy = 0.0f32;
    
    let mut xbox_lt = 0.0f32;
    let mut xbox_rt = 0.0f32;
    let mut xbox_ls = (0.0f32, 0.0f32);
    let mut xbox_rs = (0.0f32, 0.0f32);

    // Reference rate: 250Hz (4ms)
    // We scale by (dt / 0.004) to maintain consistency with the original USB 250Hz feeling
    let time_scale = dt / 0.004;

    // Pre-calculate axis values with deadzone
    let (lx_raw, ly_raw) = apply_deadzone(s.left_x, s.left_y, deadzone_l);
    let (rx_raw, ry_raw) = apply_deadzone(s.right_x, s.right_y, deadzone_r);

    // Apply smoothing (Exponential Moving Average)
    // alpha = 0.25 means 25% new data, 75% old data. 
    // This removes high frequency jitter from BT connection.
    let alpha = 0.25f32;
    smoothed_axes[0] += alpha * (lx_raw - smoothed_axes[0]);
    smoothed_axes[1] += alpha * (ly_raw - smoothed_axes[1]);
    smoothed_axes[2] += alpha * (rx_raw - smoothed_axes[2]);
    smoothed_axes[3] += alpha * (ry_raw - smoothed_axes[3]);

    let lx = smoothed_axes[0];
    let ly = smoothed_axes[1];
    let rx = smoothed_axes[2];
    let ry = smoothed_axes[3];

    // Touchpad Delta Calculation (Smoothed)
    let mut target_dx = 0.0f32;
    let mut target_dy = 0.0f32;

    if s.touch_active && *last_touch_active {
        // Calculate raw delta
        let dx_raw = s.touch_x as i32 - *last_touch_x as i32;
        let dy_raw = s.touch_y as i32 - *last_touch_y as i32;
        
        // Filter huge jumps (finger lift/place)
        if dx_raw.abs() < 500 && dy_raw.abs() < 500 {
            // Sensitivity Scaling
            // Factor 0.02 makes it manageable with standard sensitivity range (1-100)
            let factor = 0.02f32; 
            target_dx = dx_raw as f32 * sens_touchpad * factor;
            target_dy = dy_raw as f32 * sens_touchpad * factor;
        }
    } else if !s.touch_active {
        // Reset smoothing momentum immediately on lift-off
        smoothed_touch.0 = 0.0;
        smoothed_touch.1 = 0.0;
    }
    
    *last_touch_x = s.touch_x;
    *last_touch_y = s.touch_y;
    *last_touch_active = s.touch_active;

    // Apply Smoothing (Exponential Moving Average) - Match Stick Alpha
    let alpha = 0.25f32;
    smoothed_touch.0 += alpha * (target_dx - smoothed_touch.0);
    smoothed_touch.1 += alpha * (target_dy - smoothed_touch.1);

    let touch_dx = smoothed_touch.0;
    let touch_dy = smoothed_touch.1;

    for m in mappings {
        if m.source.is_axis() {
            let (ax, ay) = match m.source {
                crate::mapping::PhysicalButton::LeftStick => (lx, ly),
                crate::mapping::PhysicalButton::RightStick => (rx, ry),
                crate::mapping::PhysicalButton::L2 => (s.l2, 0.0),
                crate::mapping::PhysicalButton::R2 => (s.r2, 0.0),
                crate::mapping::PhysicalButton::Touchpad => (0.0, 0.0), // Handled specifically
                _ => (0.0, 0.0)
            };
            // Apply axis mappings
            for t in &m.targets {
                match t {
                    MappingTarget::MouseMove { .. } => {
                        if m.source == crate::mapping::PhysicalButton::Touchpad {
                            mouse_dx += touch_dx;
                            mouse_dy += touch_dy;
                        } else {
                            let sens = if m.source == crate::mapping::PhysicalButton::LeftStick { sens_l } else { sens_r };
                            mouse_dx += ax * sens * time_scale;
                            mouse_dy += ay * sens * time_scale;
                        }
                    }
                    MappingTarget::MouseScroll { speed } => {
                        // Touchpad delta is raw (e.g. 100), stick is 0.0-1.0. Scale touchpad WAY down.
                        let val = if m.source == crate::mapping::PhysicalButton::Touchpad { touch_dy * 0.05 } else { ay };
                        scroll_dy -= val * speed * time_scale; 
                    }
                    MappingTarget::XboxLT => {
                        xbox_lt = xbox_lt.max(ax);
                    }
                    MappingTarget::XboxRT => {
                        xbox_rt = xbox_rt.max(ax);
                    }
                    MappingTarget::XboxLS => {
                        xbox_ls = (ax, ay);
                    }
                    MappingTarget::XboxRS => {
                        xbox_rs = (ax, ay);
                    }
                    _ => {}
                }
            }
        } else if m.source.get_value(s) {
            for t in &m.targets {
                match t {
                    MappingTarget::Xbox(bit) => {
                        raw_buttons |= bit;
                    }
                    MappingTarget::XboxLT => {
                        xbox_lt = 1.0;
                    }
                    MappingTarget::XboxRT => {
                        xbox_rt = 1.0;
                    }
                    MappingTarget::Keyboard(vk) => {
                        current_keys.insert(*vk);
                    }
                    MappingTarget::Mouse(btn) => {
                        current_mouse.insert(*btn);
                    }
                    _ => {}
                }
            }
        }
    }

    gamepad.buttons = vigem_client::XButtons(raw_buttons);
    gamepad.left_trigger = (xbox_lt * 255.0) as u8;
    gamepad.right_trigger = (xbox_rt * 255.0) as u8;
    gamepad.thumb_lx = (xbox_ls.0 * 32767.0) as i16;
    gamepad.thumb_ly = (-xbox_ls.1 * 32767.0) as i16; 
    gamepad.thumb_rx = (xbox_rs.0 * 32767.0) as i16;
    gamepad.thumb_ry = (-xbox_rs.1 * 32767.0) as i16; 

    if state_changed {
        let _ = target.update(&gamepad);
    }

    // Keyboard Emulation
    for vk in &current_keys {
        if !active_keys.contains(vk) {
            unsafe { send_key(*vk, true); }
        }
    }
    for vk in active_keys.iter() {
        if !current_keys.contains(vk) {
            unsafe { send_key(*vk, false); }
        }
    }
    *active_keys = current_keys;

    // Mouse Buttons
    for btn in &current_mouse {
        if !active_mouse.contains(btn) {
            unsafe { send_mouse(*btn, true); }
        }
    }
    for btn in active_mouse.iter() {
        if !current_mouse.contains(btn) {
            unsafe { send_mouse(*btn, false); }
        }
    }
    *active_mouse = current_mouse;

    // Mouse Movement with Accumulation
    mouse_acc.0 += mouse_dx;
    mouse_acc.1 += mouse_dy;

    let move_x = mouse_acc.0.trunc() as i32;
    let move_y = mouse_acc.1.trunc() as i32;

    if move_x != 0 || move_y != 0 {
        mouse_acc.0 -= move_x as f32;
        mouse_acc.1 -= move_y as f32;
        unsafe {
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: move_x,
                        dy: move_y,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_MOVE,
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }

    // Mouse Scroll with Accumulation
    *scroll_acc += scroll_dy;
    let scroll_ticks = (scroll_acc.abs() / 1.0).floor() as i32;
    
    if scroll_ticks > 0 {
        let direction = if *scroll_acc > 0.0 { 1 } else { -1 };
        let move_scroll = scroll_ticks * direction;
        *scroll_acc -= move_scroll as f32;
        
        unsafe {
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: (move_scroll * 120) as u32,
                        dwFlags: MOUSEEVENTF_WHEEL,
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }
}

unsafe fn send_key(vk: u16, down: bool) {
    let scancode = MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC);
    
    let mut flags = if down { KEYBD_EVENT_FLAGS(0) } else { KEYEVENTF_KEYUP };
    if scancode > 0 {
        flags |= KEYEVENTF_SCANCODE;
    }
    
    // Some keys need extended flag (arrows, numpad enter, etc)
    if (vk >= 33 && vk <= 46) || (vk >= 91 && vk <= 93) || (vk >= 106 && vk <= 111) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scancode as u16,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
}

unsafe fn send_mouse(btn: u8, down: bool) {
    let flags = match (btn, down) {
        (0, true) => MOUSEEVENTF_LEFTDOWN,
        (0, false) => MOUSEEVENTF_LEFTUP,
        (1, true) => MOUSEEVENTF_MIDDLEDOWN,
        (1, false) => MOUSEEVENTF_MIDDLEUP,
        (2, true) => MOUSEEVENTF_RIGHTDOWN,
        (2, false) => MOUSEEVENTF_RIGHTUP,
        _ => return,
    };
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0, dy: 0, mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            }
        }
    };
    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
}

