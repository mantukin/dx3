#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, Manager, WindowBuilder, WindowUrl};
use std::thread;
use serde::Deserialize;

mod state;
mod worker;
mod dualsense; 
mod hidhide;   
mod mapping;   
mod crc;       
mod config;

use state::SharedState;
use config::AppConfig;
use worker::controller_thread;

// --- Helper Functions ---

fn create_main_window(app: &tauri::AppHandle) {
    let _ = WindowBuilder::new(
        app,
        "main",
        WindowUrl::App("index.html".into())
    )
    .title("Dx3 Controller")
    .inner_size(800.0, 800.0)
    .resizable(false)
    .fullscreen(false)
    .center()
    .visible(false) // Start hidden to prevent white flash
    .build();
}

fn show_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let state: tauri::State<Arc<Mutex<SharedState>>> = app.state();
        state.lock().unwrap().ui_visible = true;
    } else {
        create_main_window(app);
        let state: tauri::State<Arc<Mutex<SharedState>>> = app.state();
        state.lock().unwrap().ui_visible = true;
    }
}

// --- Commands ---

fn save_config_internal(s: &SharedState, persist_profile: bool) {
    // 1. Save to global config (Always save app state/active profile name)
    AppConfig::save_internal(
        s.hide_controller,
        s.start_minimized,
        s.mappings.clone(),
        s.deadzone_left,
        s.deadzone_right,
        s.current_profile_name.clone(),
        s.mouse_sens_left,
        s.mouse_sens_right,
        s.mouse_sens_touchpad,
        s.rgb_r,
        s.rgb_g,
        s.rgb_b,
        s.rgb_brightness,
        s.show_battery_led,
        s.trigger_l2_mode,
        s.trigger_l2_start,
        s.trigger_l2_force,
        s.trigger_r2_mode,
        s.trigger_r2_start,
        s.trigger_r2_force,
        s.player_led_brightness,
    );

    // 2. Only save to specific profile JSON if explicitly requested (Autosave changes)
    if persist_profile && !s.current_profile_name.is_empty() {
        let profile = crate::config::Profile {
            mappings: s.mappings.clone(),
            deadzone_left: s.deadzone_left,
            deadzone_right: s.deadzone_right,
            mouse_sens_left: s.mouse_sens_left,
            mouse_sens_right: s.mouse_sens_right,
            mouse_sens_touchpad: s.mouse_sens_touchpad,
            rgb_r: s.rgb_r,
            rgb_g: s.rgb_g,
            rgb_b: s.rgb_b,
            rgb_brightness: s.rgb_brightness,
            show_battery_led: s.show_battery_led,
            trigger_l2_mode: s.trigger_l2_mode,
            trigger_l2_start: s.trigger_l2_start,
            trigger_l2_force: s.trigger_l2_force,
            trigger_r2_mode: s.trigger_r2_mode,
            trigger_r2_start: s.trigger_r2_start,
            trigger_r2_force: s.trigger_r2_force,
            player_led_brightness: s.player_led_brightness,
        };
        AppConfig::save_profile(&s.current_profile_name, &profile);
    }
}

#[tauri::command]
fn trigger_driver_refresh(state: tauri::State<Arc<Mutex<SharedState>>>) {
    let mut s = state.lock().unwrap();
    s.should_reinit = true;
    s.status = "Refreshing drivers...".to_string();
}

#[tauri::command]
fn resume_scanning(state: tauri::State<Arc<Mutex<SharedState>>>) {
    let mut s = state.lock().unwrap();
    s.is_paused = false;
    s.status = "Searching...".to_string();
}

#[tauri::command]
fn disconnect_controller(state: tauri::State<Arc<Mutex<SharedState>>>) {
    state.lock().unwrap().should_disconnect = true;
}

#[tauri::command]
fn set_show_battery_led(state: tauri::State<Arc<Mutex<SharedState>>>, val: bool) {
    let mut s = state.lock().unwrap();
    s.show_battery_led = val;
    s.should_send_leds = true;
    save_config_internal(&s, true);
}

#[tauri::command]
fn set_player_led_brightness(state: tauri::State<Arc<Mutex<SharedState>>>, val: u8) {
    let mut s = state.lock().unwrap();
    s.player_led_brightness = val;
    s.should_send_leds = true;
    save_config_internal(&s, true);
}

#[tauri::command]
fn set_rgb(state: tauri::State<Arc<Mutex<SharedState>>>, r: u8, g: u8, b: u8, brightness: u8) {
    let mut s = state.lock().unwrap();
    s.rgb_r = r;
    s.rgb_g = g;
    s.rgb_b = b;
    s.rgb_brightness = brightness;
    s.should_send_leds = true;
    save_config_internal(&s, true);
}

#[tauri::command]
fn set_trigger_l2(state: tauri::State<Arc<Mutex<SharedState>>>, mode: u8, start: u8, force: u8) {
    let mut s = state.lock().unwrap();
    s.trigger_l2_mode = mode;
    s.trigger_l2_start = start;
    s.trigger_l2_force = force;
    s.should_send_triggers = true;
    save_config_internal(&s, true);
}

#[tauri::command]
fn set_trigger_r2(state: tauri::State<Arc<Mutex<SharedState>>>, mode: u8, start: u8, force: u8) {
    let mut s = state.lock().unwrap();
    s.trigger_r2_mode = mode;
    s.trigger_r2_start = start;
    s.trigger_r2_force = force;
    s.should_send_triggers = true;
    save_config_internal(&s, true);
}

#[tauri::command]
fn set_deadzones(state: tauri::State<Arc<Mutex<SharedState>>>, left: f32, right: f32) {
    let mut s = state.lock().unwrap();
    s.deadzone_left = left;
    s.deadzone_right = right;
    save_config_internal(&s, true);
}

#[tauri::command]
fn set_mouse_sens(state: tauri::State<Arc<Mutex<SharedState>>>, left: f32, right: f32) {
    let mut s = state.lock().unwrap();
    s.mouse_sens_left = left;
    s.mouse_sens_right = right;
    save_config_internal(&s, true);
}

#[tauri::command]
fn set_touchpad_sens(state: tauri::State<Arc<Mutex<SharedState>>>, sens: f32) {
    let mut s = state.lock().unwrap();
    s.mouse_sens_touchpad = sens;
    save_config_internal(&s, true);
}

#[derive(Deserialize)]
pub struct ManualParams {
    pub report_id: u8,
    pub flag_off: usize,
    pub rgb_off: usize,
    pub player_led: u8,
    pub pled_bright: u8,
    pub pled_bright_off: usize,
    pub bt_flags: u8,
    pub bt_flags2: u8,
    pub bt_len: usize,
    pub as_feature: bool,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[tauri::command]
fn get_initial_state(state: tauri::State<Arc<Mutex<SharedState>>>) -> String {
    let s = state.lock().unwrap();
    serde_json::to_string(&*s).unwrap_or("{}".to_string())
}

#[tauri::command]
fn is_dev() -> bool {
    #[cfg(debug_assertions)]
    return true;
    #[cfg(not(debug_assertions))]
    return false;
}

#[tauri::command]
fn toggle_debug(_state: tauri::State<Arc<Mutex<SharedState>>>) {
    #[cfg(debug_assertions)]
    {
        let mut s = _state.lock().unwrap();
        s.debug_active = !s.debug_active;
    }
}

#[tauri::command]
fn set_hide_controller(state: tauri::State<Arc<Mutex<SharedState>>>, hide: bool) {
    let mut s = state.lock().unwrap();
    s.hide_controller = hide;
    save_config_internal(&s, false); // Don't save to profile, global setting
}

#[tauri::command]
fn set_start_minimized(state: tauri::State<Arc<Mutex<SharedState>>>, val: bool) {
    let mut s = state.lock().unwrap();
    s.start_minimized = val;
    save_config_internal(&s, false); // Global setting
}

#[tauri::command]
fn set_fuzzer_active(state: tauri::State<Arc<Mutex<SharedState>>>, val: bool) {
    let mut s = state.lock().unwrap();
    s.fuzzer_active = val;
    if val {
        s.fuzzer_step = 0;
        s.fuzzer_log = "Starting...".to_string();
    } else {
        s.fuzzer_log = "Stopped.".to_string();
    }
}

#[tauri::command]
fn set_sweep_active(state: tauri::State<Arc<Mutex<SharedState>>>, val: bool) {
    let mut s = state.lock().unwrap();
    s.sweep_active = val;
    if val {
        s.fuzzer_step = 0;
        s.fuzzer_log = "Sweeping...".to_string();
    }
}

#[tauri::command]
fn set_sweep_speed(state: tauri::State<Arc<Mutex<SharedState>>>, val: u64) {
    let mut s = state.lock().unwrap();
    s.sweep_timeout_ms = val;
}

#[tauri::command]
fn set_disable_periodic(state: tauri::State<Arc<Mutex<SharedState>>>, val: bool) {
    let mut s = state.lock().unwrap();
    s.disable_periodic = val;
}

#[tauri::command]
fn set_crc_seed(state: tauri::State<Arc<Mutex<SharedState>>>, val: u8) {
    let mut s = state.lock().unwrap();
    s.crc_seed_idx = val;
}

#[tauri::command]
fn set_manual_params(state: tauri::State<Arc<Mutex<SharedState>>>, params: ManualParams) {
    let mut s = state.lock().unwrap();
    s.manual_report_id = params.report_id;
    s.manual_flag_offset = params.flag_off;
    s.manual_rgb_offset = params.rgb_off;
    s.manual_player_led = params.player_led;
    s.manual_pled_bright = params.pled_bright;
    s.manual_pled_bright_off = params.pled_bright_off;
    s.bt_flag_val = params.bt_flags;
    s.bt_flag_val2 = params.bt_flags2;
    s.manual_bt_len = params.bt_len;
    s.send_as_feature = params.as_feature;
    s.manual_r = params.r;
    s.manual_g = params.g;
    s.manual_b = params.b;
}

#[tauri::command]
fn trigger_manual_send(state: tauri::State<Arc<Mutex<SharedState>>>) {
    state.lock().unwrap().should_send_manual = true;
}

#[tauri::command]
fn set_pinpoint_params(state: tauri::State<Arc<Mutex<SharedState>>>, offset: usize, value: u8) {
    let mut s = state.lock().unwrap();
    s.pinpoint_offset = offset;
    s.pinpoint_value = value;
}

#[tauri::command]
fn trigger_pinpoint_send(state: tauri::State<Arc<Mutex<SharedState>>>) {
    state.lock().unwrap().should_send_pinpoint = true;
}

#[tauri::command]
fn trigger_protocol_scan(state: tauri::State<Arc<Mutex<SharedState>>>) {
    let mut s = state.lock().unwrap();
    s.protocol_scan_active = true;
    s.protocol_log = "Scanning... Please wait.".to_string();
}

#[tauri::command]
fn update_mappings(state: tauri::State<Arc<Mutex<SharedState>>>, mappings: Vec<crate::mapping::ButtonMapping>) {
    let mut s = state.lock().unwrap();
    s.mappings = mappings;
    s.mappings_changed = true;
    save_config_internal(&s, true);
}

#[tauri::command]
fn reset_mappings(state: tauri::State<Arc<Mutex<SharedState>>>) {
    let mut s = state.lock().unwrap();
    s.mappings = AppConfig::default_mappings();
    s.mappings_changed = true;
    s.current_profile_name = "Default".to_string();
    save_config_internal(&s, true);
}

#[tauri::command]
fn get_profiles() -> Vec<String> {
    AppConfig::list_profiles()
}

#[tauri::command]
fn save_profile(state: tauri::State<Arc<Mutex<SharedState>>>, name: String) {
    let mut s = state.lock().unwrap();
    s.current_profile_name = name;
    save_config_internal(&s, true);
}

#[tauri::command]
fn load_profile(state: tauri::State<Arc<Mutex<SharedState>>>, name: String) {
    let mut s = state.lock().unwrap();
    
    // Special handling for "Default" if it doesn't exist on disk yet
    if name == "Default" {
        // Try to load, if fails, reset to hardcoded defaults
        if let Some(profile) = AppConfig::load_profile(&name) {
            apply_profile_to_state(&mut s, profile);
        } else {
            s.mappings = AppConfig::default_mappings();
            // Reset crucial settings to defaults
            s.deadzone_left = 0.1; s.deadzone_right = 0.1;
            s.mouse_sens_left = 25.0; s.mouse_sens_right = 25.0; s.mouse_sens_touchpad = 25.0;
            s.rgb_r = 0; s.rgb_g = 0; s.rgb_b = 255; s.rgb_brightness = 255;
            s.show_battery_led = false;
            s.trigger_l2_mode = 0; s.trigger_r2_mode = 0;
            s.player_led_brightness = 0;
            
            s.mappings_changed = true;
            s.should_send_leds = true;
            s.should_send_triggers = true;
        }
        s.current_profile_name = name;
        save_config_internal(&s, false); // DO NOT OVERWRITE PROFILE ON LOAD
        return;
    }

    if let Some(profile) = AppConfig::load_profile(&name) {
        apply_profile_to_state(&mut s, profile);
        s.current_profile_name = name;
        save_config_internal(&s, false); // DO NOT OVERWRITE PROFILE ON LOAD
    }
}

fn apply_profile_to_state(s: &mut SharedState, p: crate::config::Profile) {
    s.mappings = p.mappings;
    s.deadzone_left = p.deadzone_left;
    s.deadzone_right = p.deadzone_right;
    s.mouse_sens_left = p.mouse_sens_left;
    s.mouse_sens_right = p.mouse_sens_right;
    s.mouse_sens_touchpad = p.mouse_sens_touchpad;
    s.rgb_r = p.rgb_r;
    s.rgb_g = p.rgb_g;
    s.rgb_b = p.rgb_b;
    s.rgb_brightness = p.rgb_brightness;
    s.show_battery_led = p.show_battery_led;
    s.trigger_l2_mode = p.trigger_l2_mode;
    s.trigger_l2_start = p.trigger_l2_start;
    s.trigger_l2_force = p.trigger_l2_force;
    s.trigger_r2_mode = p.trigger_r2_mode;
    s.trigger_r2_start = p.trigger_r2_start;
    s.trigger_r2_force = p.trigger_r2_force;
    s.player_led_brightness = p.player_led_brightness;

    s.mappings_changed = true;
    s.should_send_leds = true;
    s.should_send_triggers = true;
}

#[tauri::command]
fn delete_profile(name: String) {
    AppConfig::delete_profile(&name);
}

#[tauri::command]
fn get_image_asset(name: String) -> Vec<u8> {
    match name.as_str() {
        "dualsense.webp" => include_bytes!("../assets/dualsense.webp").to_vec(),
        "analog_stick_head.webp" => include_bytes!("../assets/analog_stick_head.webp").to_vec(),
        "LED.webp" => include_bytes!("../assets/LED.webp").to_vec(),
        "github_icon.webp" => include_bytes!("../assets/github_icon.webp").to_vec(),
        _ => Vec::new(),
    }
}

fn main() {
    // Initialize logger: Suppress noisy warnings from TAO (windowing) and WRY (webview)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("tao", log::LevelFilter::Error)
        .filter_module("wry", log::LevelFilter::Error)
        .init();

    let config = AppConfig::load();
    let state = Arc::new(Mutex::new(SharedState::new(&config)));
    let state_clone = state.clone();

    // Tray Setup
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show/Hide");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    // Global Signal Handler (Ctrl+C, SIGTERM)
    let state_for_signal = state.clone();
    let _ = ctrlc::set_handler(move || {
        let mut s = state_for_signal.lock().unwrap();
        s.should_exit = true;
        // Release lock and wait
        drop(s);
        std::thread::sleep(std::time::Duration::from_millis(300));
        std::process::exit(0);
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_window(app);
        }))
        .manage(state)
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                show_window(app);
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                let state: tauri::State<Arc<Mutex<SharedState>>> = app.state();
                match id.as_str() {
                    "quit" => {
                        state.lock().unwrap().should_exit = true;
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        std::process::exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.close(); // Destroy to free RAM
                                state.lock().unwrap().ui_visible = false;
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                                state.lock().unwrap().ui_visible = true;
                            }
                        } else {
                            show_window(app);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { .. } => {
                // Allow the window to close (destroying webview)
                let app_handle = event.window().app_handle();
                let state: tauri::State<Arc<Mutex<SharedState>>> = app_handle.state();
                state.lock().unwrap().ui_visible = false;
            }
            _ => {}
        })
        .setup(move |app| {
            let app_handle = app.handle();
            let app_handle_for_worker = app_handle.clone();
            
            // Start Background Worker
            thread::spawn(move || {
                controller_thread(state_clone, app_handle_for_worker);
            });
            
            // Initial Window Logic
            if config.start_minimized {
                // If starting minimized, DESTROY the auto-created window so it doesn't consume RAM
                // and so main.js doesn't run and force-show it.
                if let Some(window) = app_handle.get_window("main") {
                    let _ = window.close();
                }
            }
            // Note: If NOT minimized, we do nothing here. The window is created hidden (via tauri.conf.json),
            // and main.js calls appWindow.show() only after assets are loaded.

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_initial_state, toggle_debug, is_dev, set_hide_controller, set_start_minimized,
            trigger_driver_refresh,
            set_fuzzer_active, set_sweep_active, set_sweep_speed, set_disable_periodic, set_crc_seed,
            set_manual_params, trigger_manual_send,
            set_pinpoint_params, trigger_pinpoint_send, trigger_protocol_scan,
            update_mappings, reset_mappings,
            set_deadzones, set_mouse_sens, set_touchpad_sens, set_rgb, set_show_battery_led, set_player_led_brightness,
            set_trigger_l2, set_trigger_r2, disconnect_controller, resume_scanning,
            get_profiles, save_profile, load_profile, delete_profile,
            get_image_asset
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                // Keep the app running in the background when the window is closed
                api.prevent_exit();
            }
            _ => {}
        });
}
