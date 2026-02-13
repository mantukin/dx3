use hidapi::HidDevice;
use crate::crc;

pub fn send_dualsense_output(
    device: &HidDevice, 
    is_bt: bool, 
    red: u8, green: u8, blue: u8, 
    player_led_mask: u8, 
    player_led_brightness: u8, // 0=High, 1=Med, 2=Low
    seq: u8,
    // Adaptive Triggers
    l2_mode: u8, l2_start: u8, l2_force: u8,
    r2_mode: u8, r2_start: u8, r2_force: u8,
) {
    let mut report = [0u8; 78];
    
    // USB: Player @ 44, RGB @ 45, R2 Trigger @ 11, L2 Trigger @ 22
    // BT:  Player @ 45, RGB @ 46, R2 Trigger @ 12, L2 Trigger @ 23 (+1 shift)
    let (report_id, offset_player_led, offset_rgb, offset_r2, offset_l2) = if is_bt {
        (0x31, 45, 46, 12, 23)
    } else {
        (0x02, 44, 45, 11, 22)
    };

    report[0] = report_id;
    if is_bt {
        // BT Header - IMPORTANT: 0x02 in low nibble is required for LED work!
        report[1] = (seq << 4) | 0x02; 
        // Flags: 0x04 = triggers, 0x08 = LED
        report[2] = 0xFF;  // All flags for triggers and LED
        report[3] = 0x15; 
        report[4] = 0x00; // No vibration
        report[5] = 0x00;
    } else {
        // USB Flags - using same values as Manual Override
        // Byte 1: trigger flags + LED control
        report[1] = 0xF7;  
        // Byte 2: LED flags
        report[2] = 0x15;  
    } 
    
    // R2 Trigger (Right)
    report[offset_r2] = r2_mode;
    report[offset_r2 + 1] = r2_start;  // Start position
    report[offset_r2 + 2] = r2_force;  // Force
    
    // L2 Trigger (Left)
    report[offset_l2] = l2_mode;
    report[offset_l2 + 1] = l2_start;
    report[offset_l2 + 2] = l2_force;
    
    // Player LED Brightness Flag: Byte 39 (USB) / 40 (BT)
    // Bit 0x01 = apply player_led_brightness value
    // Bit 0x02 = fade animation
    let offset_pled_flags = if is_bt { 40 } else { 39 };
    report[offset_pled_flags] = 0x01; // Enable brightness control
    
    // Lightbar Setup: Byte 42 (USB) / 43 (BT)
    // Bit 0x02 = LIGHT_OUT (enable lightbar output)
    let offset_lightbar_setup = if is_bt { 43 } else { 42 };
    report[offset_lightbar_setup] = 0x02; // LIGHT_OUT
    
    // LED Data
    // Byte 43 (USB) / 44 (BT) = player_led_brightness (0=High, 1=Med, 2=Low)
    if offset_player_led > 0 {
        report[offset_player_led - 1] = player_led_brightness;
    }
    // Byte 44 (USB) / 45 (BT) = player_led_mask
    // Bit 0x20 = immediate brightness (no fade-in)
    report[offset_player_led] = player_led_mask | 0x20;
    report[offset_rgb] = red;
    report[offset_rgb + 1] = green;
    report[offset_rgb + 2] = blue;

    if is_bt {
        // CRC32 considering BT phantom header 0xA2
        let checksum = crc::crc32_bt(&report[0..74]);
        report[74] = (checksum & 0xFF) as u8;
        report[75] = ((checksum >> 8) & 0xFF) as u8;
        report[76] = ((checksum >> 16) & 0xFF) as u8;
        report[77] = ((checksum >> 24) & 0xFF) as u8;
        
        let _ = device.write(&report);
    } else {
        let _ = device.write(&report[0..64]);
    }
}

pub fn send_power_off(device: &hidapi::HidDevice, is_bt: bool, seq: u8) {
    if is_bt {
        let mut report = [0u8; 78];
        report[0] = 0x31;
        
        // Byte 1: 0x02 (HID Output) | 0x40 (Disconnect bit) | Seq
        report[1] = (seq << 4) | 0x02 | 0x40; 
        
        // Bytes 2 and 3: Activation masks forcing the controller to accept the packet
        report[2] = 0xF7; 
        report[3] = 0x15; 
        
        // Bytes 4 and 5: Vibration. STRICTLY 0x00.
        report[4] = 0x00; 
        report[5] = 0x00;
        
        // Clear the rest
        for i in 6..74 { report[i] = 0; }
        
        let checksum = crc::crc32_bt(&report[0..74]);
        report[74] = (checksum & 0xFF) as u8;
        report[75] = ((checksum >> 8) & 0xFF) as u8;
        report[76] = ((checksum >> 16) & 0xFF) as u8;
        report[77] = ((checksum >> 24) & 0xFF) as u8;
        
        let _ = device.write(&report);
    }
}

pub fn send_raw_output(
    device: &HidDevice, 
    report_id: u8, 
    flag_off: usize, 
    rgb_off: usize, 
    r: u8, g: u8, b: u8, 
    seq: u8, 
    _crc_mode: u8, 
    player_val: u8, 
    pled_bright: u8,
    pled_bright_off: usize,
    flag_val: u8, 
    flag_val2: u8, 
    _bt_len: usize, 
    as_feature: bool
) -> Result<(usize, String), String> {
    let mut report = [0u8; 600]; 
    report[0] = report_id;

    // BT Headers if 0x31 or 0x11 (DS4)
    if report_id == 0x31 || report_id == 0x11 {
        // IMPORTANT: 0x02 in low nibble required for LED work!
        report[1] = (seq << 4) | 0x02; 
    }

    // Set Flags (important for BT LED activation)
    if flag_off < 590 {
        report[flag_off] = flag_val;
        // BT also needs flags in Byte 3 (Player LED / LED activation)
        // But NOT in Byte 4 (Vibration)!
        if report_id == 0x31 && flag_off == 2 {
            report[3] = flag_val2;
            report[4] = 0x00; // Force no vibration for raw test
        }
    }

    // Set RGB & Player & Brightness
    if rgb_off < 595 {
        report[rgb_off] = r;
        report[rgb_off + 1] = g;
        report[rgb_off + 2] = b;
        
        if rgb_off > 0 {
            // Add bit 0x20 for immediate brightness
            report[rgb_off - 1] = player_val | 0x20;
        }
    }
    
    // Set Player Brightness explicitly
    if pled_bright_off > 0 && pled_bright_off < 599 {
        report[pled_bright_off] = pled_bright;
        
        // Set brightness enable flag in Byte 39 (USB) / 40 (BT)
        let offset_pled_flags = if report_id == 0x31 { 40 } else { 39 };
        report[offset_pled_flags] |= 0x01; // Enable brightness control
    }

    // CRC for BT
    if report_id == 0x31 {
        let checksum = crc::crc32_bt(&report[0..74]);
        
        report[74] = (checksum & 0xFF) as u8;
        report[75] = ((checksum >> 8) & 0xFF) as u8;
        report[76] = ((checksum >> 16) & 0xFF) as u8;
        report[77] = ((checksum >> 24) & 0xFF) as u8;
    }

    let len = if report_id == 0x31 { 78 } else { 64 };
    let slice = &report[0..len];
    
    // Hex String Generation
    let hex_str = slice.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ");

    if as_feature {
        match device.send_feature_report(slice) {
            Ok(_) => Ok((slice.len(), hex_str)),
            Err(e) => Err(format!("{} | Hex: {}", e, hex_str))
        }
    } else {
        match device.write(slice) {
            Ok(n) => Ok((n, hex_str)),
            Err(e) => Err(format!("{} | Hex: {}", e, hex_str))
        }
    }
}

/// Initializes DualSense LED controller via "wake-up" packet.
/// After BT reconnection, controller requires a special signal
/// with flags 0xFF/0xFF/0xFF in bytes 2-4 (like in step 2 of RGB Sweep).
pub fn send_led_init(device: &HidDevice, seq: u8, target_pled: u8, r: u8, g: u8, b: u8) {
    // 1. Wake-up packet with max flags
    send_wakeup_packet_bt(device, seq);
    
    // 2. Short pause
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // 3. Regular LED packet with desired settings
    send_led_packet_bt(device, seq.wrapping_add(1), target_pled, r, g, b);
}

/// Wake-up packet: flags 0xFF in bytes 2, 3, 4 (mimics RGB Sweep step 2)
fn send_wakeup_packet_bt(device: &HidDevice, seq: u8) {
    let mut report = [0u8; 78];
    report[0] = 0x31;
    report[1] = (seq << 4) | 0x02;
    // Key: bytes 2-4 = 0xFF (max activation flags)
    report[2] = 0xFF;
    report[3] = 0xFF;
    report[4] = 0xFF;
    
    let checksum = crc::crc32_bt(&report[0..74]);
    report[74] = (checksum & 0xFF) as u8;
    report[75] = ((checksum >> 8) & 0xFF) as u8;
    report[76] = ((checksum >> 16) & 0xFF) as u8;
    report[77] = ((checksum >> 24) & 0xFF) as u8;
    
    let _ = device.write(&report);
}

fn send_led_packet_bt(device: &HidDevice, seq: u8, pled: u8, r: u8, g: u8, b: u8) {
    let mut report = [0u8; 78];
    report[0] = 0x31;
    report[1] = (seq << 4) | 0x02;
    report[2] = 0xFF;  // Feature flags
    report[3] = 0x15;  // LED flags
    report[4] = 0x00;  // No vibration
    
    // Player LED @ offset 45, RGB @ offset 46-48
    report[45] = pled;
    report[46] = r;
    report[47] = g;
    report[48] = b;
    
    let checksum = crc::crc32_bt(&report[0..74]);
    report[74] = (checksum & 0xFF) as u8;
    report[75] = ((checksum >> 8) & 0xFF) as u8;
    report[76] = ((checksum >> 16) & 0xFF) as u8;
    report[77] = ((checksum >> 24) & 0xFF) as u8;
    
    let _ = device.write(&report);
}

/// USB Wake-up packet: 0xFF flags in bytes 1-2 to init LED + rumble
pub fn send_led_init_usb(device: &HidDevice, target_pled: u8, r: u8, g: u8, b: u8) {
    // Wake-up packet with max flags
    let mut report = [0u8; 64];
    report[0] = 0x02;
    report[1] = 0xFF;  // All flags (triggers + LED)
    report[2] = 0xFF;  // All LED flags
    // Bytes 3-4: rumble (short pulse)
    report[3] = 0x20;  // Small rumble left
    report[4] = 0x20;  // Small rumble right
    
    // Player LED @ offset 44, RGB @ offset 45-47
    report[44] = target_pled;
    report[45] = r;
    report[46] = g;
    report[47] = b;
    
    let _ = device.write(&report);
}
