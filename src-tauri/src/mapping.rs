use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GamepadState {
    pub left_x: f32,
    pub left_y: f32,
    pub right_x: f32,
    pub right_y: f32,
    pub l2: f32,
    pub r2: f32,
    pub btn_cross: bool,
    pub btn_circle: bool,
    pub btn_square: bool,
    pub btn_triangle: bool,
    pub btn_l1: bool,
    pub btn_r1: bool,
    pub btn_l3: bool,
    pub btn_r3: bool,
    pub btn_options: bool,
    pub btn_share: bool,
    pub btn_ps: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub btn_touchpad: bool,
    pub btn_mute: bool,
    pub touch_x: u16,
    pub touch_y: u16,
    pub touch_active: bool,
    pub battery: u8, // 0-100
    pub is_charging: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhysicalButton {
    Cross, Circle, Square, Triangle,
    L1, R1, L3, R3,
    Options, Share, PS, Touchpad, TouchpadLeft, TouchpadRight, Mute,
    DpadUp, DpadDown, DpadLeft, DpadRight,
    LeftStick, RightStick, L2, R2,
}

impl PhysicalButton {
    pub fn is_axis(&self) -> bool {
        match self {
            Self::LeftStick | Self::RightStick | Self::L2 | Self::R2 | Self::Touchpad => true,
            _ => false
        }
    }

    pub fn get_value(&self, state: &GamepadState) -> bool {
        match self {
            Self::Cross => state.btn_cross,
            Self::Circle => state.btn_circle,
            Self::Square => state.btn_square,
            Self::Triangle => state.btn_triangle,
            Self::L1 => state.btn_l1,
            Self::R1 => state.btn_r1,
            Self::L3 => state.btn_l3,
            Self::R3 => state.btn_r3,
            Self::Options => state.btn_options,
            Self::Share => state.btn_share,
            Self::PS => state.btn_ps,
            Self::Touchpad => state.btn_touchpad,
            Self::TouchpadLeft => state.btn_touchpad && state.touch_x < 960,
            Self::TouchpadRight => state.btn_touchpad && state.touch_x >= 960,
            Self::Mute => state.btn_mute,
            Self::DpadUp => state.dpad_up,
            Self::DpadDown => state.dpad_down,
            Self::DpadLeft => state.dpad_left,
            Self::DpadRight => state.dpad_right,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MappingTarget {
    Xbox(u16),      // Bitmask from vigem_client::XButtons
    XboxLT,         // Left Trigger
    XboxRT,         // Right Trigger
    XboxLS,         // Left Stick (Analog)
    XboxRS,         // Right Stick (Analog)
    Keyboard(u16),  // Virtual Key Code (VK_*)
    Mouse(u8),      // 0: Left, 1: Right, 2: Middle
    MouseMove { x_speed: f32, y_speed: f32 },
    MouseScroll { speed: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonMapping {
    pub source: PhysicalButton,
    pub targets: Vec<MappingTarget>,
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            left_x: 0.0, left_y: 0.0, right_x: 0.0, right_y: 0.0,
            l2: 0.0, r2: 0.0,
            btn_cross: false, btn_circle: false, btn_square: false, btn_triangle: false,
            btn_l1: false, btn_r1: false, btn_l3: false, btn_r3: false,
            btn_options: false, btn_share: false, btn_ps: false,
            dpad_up: false, dpad_down: false, dpad_left: false, dpad_right: false,
            btn_touchpad: false,
            btn_mute: false,
            touch_x: 0, touch_y: 0, touch_active: false,
            battery: 0,
            is_charging: false,
        }
    }
}

pub fn normalize_axis(val: u8) -> f32 {
    (val as f32 - 128.0) / 128.0
}

pub fn normalize_trigger(val: u8) -> f32 {
    val as f32 / 255.0
}

// DualSense Parsing
pub fn parse_dualsense(report: &[u8], is_bt: bool) -> Option<GamepadState> {
    let report_id = report[0];
    
    if is_bt {
        // Bluetooth
        if report_id == 0x01 {
            // Simple Mode (Classic HID) - always use simple parser regardless of length (windows padding)
            return Some(parse_dualsense_simple(report));
        }
        if report_id == 0x31 && report.len() >= 12 {
            // Native Mode
            return Some(parse_dualsense_bt(report));
        }
    } else {
        // USB
        if report_id == 0x01 {
            // USB Native is usually 64 bytes
            return Some(parse_dualsense_usb(report));
        }
    }

    None
}

fn parse_dualsense_simple(report: &[u8]) -> GamepadState {
    let mut state = GamepadState::default();
    
    // Simple Report Layout (Standard HID)
    // 1: LX, 2: LY, 3: RX, 4: RY
    state.left_x = normalize_axis(report[1]);
    state.left_y = normalize_axis(report[2]);
    state.right_x = normalize_axis(report[3]);
    state.right_y = normalize_axis(report[4]);

    // Byte 5: D-Pad (Low 4) + Face Buttons (High 4)
    let dpad = report[5] & 0x0F;
    if dpad != 8 {
        match dpad {
            0 => state.dpad_up = true,
            1 => { state.dpad_up = true; state.dpad_right = true; },
            2 => state.dpad_right = true,
            3 => { state.dpad_right = true; state.dpad_down = true; },
            4 => state.dpad_down = true,
            5 => { state.dpad_down = true; state.dpad_left = true; },
            6 => state.dpad_left = true,
            7 => { state.dpad_left = true; state.dpad_up = true; },
            _ => {}
        }
    }

    let face = report[5] >> 4;
    state.btn_square = (face & 0x1) != 0;
    state.btn_cross = (face & 0x2) != 0;
    state.btn_circle = (face & 0x4) != 0;
    state.btn_triangle = (face & 0x8) != 0;

    // Byte 6: Misc
    let misc = report[6];
    state.btn_l1 = (misc & 0x01) != 0;
    state.btn_r1 = (misc & 0x02) != 0;
    // L2/R2 are digital in simple mode often, or mapped to Z/Rz axes later.
    // Assuming digital bits 2 and 3 for now as fallback.
    let l2_dig = (misc & 0x04) != 0;
    let r2_dig = (misc & 0x08) != 0;
    state.l2 = if l2_dig { 1.0 } else { 0.0 };
    state.r2 = if r2_dig { 1.0 } else { 0.0 };
    
    state.btn_share = (misc & 0x10) != 0;
    state.btn_options = (misc & 0x20) != 0;
    state.btn_l3 = (misc & 0x40) != 0;
    state.btn_r3 = (misc & 0x80) != 0;

    // Byte 7: PS / Touch
    if report.len() > 7 {
        let extra = report[7];
        state.btn_ps = (extra & 0x01) != 0;
        state.btn_touchpad = (extra & 0x02) != 0;
        state.btn_mute = (extra & 0x04) != 0;
    }

    state
}

fn parse_dualsense_usb(report: &[u8]) -> GamepadState {
    let mut state = GamepadState::default();
    if report.len() < 11 { return state; }

    // Based on user debug info:
    // report[1..4] -> Sticks
    // report[5] -> L2 Analog
    // report[6] -> R2 Analog
    // report[8] -> Buttons 1 (Dpad + Face)
    // report[9] -> Buttons 2 (L1, R1, Share, Opt, L3, R3)
    // report[10] -> Buttons 3 (PS, Touchpad)

    state.left_x = normalize_axis(report[1]);
    state.left_y = normalize_axis(report[2]);
    state.right_x = normalize_axis(report[3]);
    state.right_y = normalize_axis(report[4]);
    
    state.l2 = normalize_trigger(report[5]);
    state.r2 = normalize_trigger(report[6]);

    // Buttons 1 (Index 8)
    let dpad = report[8] & 0x0F;
    if dpad != 8 {
        match dpad {
            0 => state.dpad_up = true,
            1 => { state.dpad_up = true; state.dpad_right = true; },
            2 => state.dpad_right = true,
            3 => { state.dpad_right = true; state.dpad_down = true; },
            4 => state.dpad_down = true,
            5 => { state.dpad_down = true; state.dpad_left = true; },
            6 => state.dpad_left = true,
            7 => { state.dpad_left = true; state.dpad_up = true; },
            _ => {}
        }
    }

    let face = report[8] >> 4;
    state.btn_square = (face & 0x1) != 0;
    state.btn_cross = (face & 0x2) != 0;
    state.btn_circle = (face & 0x4) != 0;
    state.btn_triangle = (face & 0x8) != 0;

    // Buttons 2 (Index 9)
    let b2 = report[9];
    state.btn_l1 = (b2 & 0x01) != 0;
    state.btn_r1 = (b2 & 0x02) != 0;
    state.btn_share = (b2 & 0x10) != 0;
    state.btn_options = (b2 & 0x20) != 0;
    state.btn_l3 = (b2 & 0x40) != 0;
    state.btn_r3 = (b2 & 0x80) != 0;

    // Buttons 3 (Index 10)
    let b3 = report[10];
    state.btn_ps = (b3 & 0x01) != 0;
    state.btn_touchpad = (b3 & 0x02) != 0;
    state.btn_mute = (b3 & 0x04) != 0;

    // Battery for DualSense USB is typically at index 53 (offset 52 if report[0] is ID)
    if report.len() >= 54 {
        let b_val = report[53];
        state.battery = ((b_val & 0x0F) * 10).min(100);
        state.is_charging = (b_val & 0x10) != 0;
    }

    state
}

fn parse_dualsense_bt(data: &[u8]) -> GamepadState {
    let mut state = GamepadState::default();

    // Data offsets for DS BT Report 0x31 (Empirically found)
    // 0: 0x31
    // 1: Seq/Unk
    // 2: LX
    // 3: LY
    // 4: RX
    // 5: RY
    // 6: L2 Analog (Found via debug grid)
    // 7: R2 Analog (Hypothesis: follows L2)
    // 8: Unknown/Padding
    // 9: Buttons 1 (Square, Cross, Circle, Triangle, DPad)
    // 10: Buttons 2 (L1, R1, L2_dig, R2_dig, Create, Options, L3, R3)
    // 11: Buttons 3 (PS, Mute, Touch)

    if data.len() < 14 { return state; }

    state.left_x = normalize_axis(data[2]);
    state.left_y = normalize_axis(data[3]); // Reverted inversion based on latest feedback
    state.right_x = normalize_axis(data[4]);
    state.right_y = normalize_axis(data[5]); // Reverted inversion
    
    // Analog Triggers
    state.l2 = normalize_trigger(data[6]);
    state.r2 = normalize_trigger(data[7]); 

    // Buttons Byte 9
    let dpad = data[9] & 0x0F;
    if dpad != 8 {
        match dpad {
            0 => state.dpad_up = true,
            1 => { state.dpad_up = true; state.dpad_right = true; },
            2 => state.dpad_right = true,
            3 => { state.dpad_right = true; state.dpad_down = true; },
            4 => state.dpad_down = true,
            5 => { state.dpad_down = true; state.dpad_left = true; },
            6 => state.dpad_left = true,
            7 => { state.dpad_left = true; state.dpad_up = true; },
            _ => {}
        }
    }

    let buttons1 = data[9] >> 4;
    state.btn_square = (buttons1 & 0x1) != 0;
    state.btn_cross = (buttons1 & 0x2) != 0;
    state.btn_circle = (buttons1 & 0x4) != 0;
    state.btn_triangle = (buttons1 & 0x8) != 0;

    // Buttons Byte 10
    let buttons2 = data[10];
    state.btn_l1 = (buttons2 & 0x01) != 0;
    state.btn_r1 = (buttons2 & 0x02) != 0;
    state.btn_share = (buttons2 & 0x10) != 0; // Create
    state.btn_options = (buttons2 & 0x20) != 0; // Options
    state.btn_l3 = (buttons2 & 0x40) != 0;
    state.btn_r3 = (buttons2 & 0x80) != 0;

    // Buttons Byte 11
    if (data[11] & 0x01) != 0 {
        state.btn_ps = true;
    }
    state.btn_mute = (data[11] & 0x04) != 0;
    state.btn_touchpad = (data[11] & 0x02) != 0;

    // Touchpad Data (DualSense BT Report 0x31)
    // Starts at byte 33.
    if data.len() >= 38 {
        // Byte 33: Touch Packet Count (usually)
        // Byte 34: Touch 1 ID & Active Flag. (Bit 7: 0 = Active, 1 = Inactive)
        let t1_info = data[34];
        let t1_active = (t1_info & 0x80) == 0;
        
        if t1_active {
            state.touch_active = true;
            // Byte 35: X Low
            // Byte 36: X High (0-3) | Y Low (4-7)
            // Byte 37: Y High
            let x_lo = data[35] as u16;
            let mid = data[36] as u16;
            let y_hi = data[37] as u16;
            
            // X: 12 bits
            let x_hi = mid & 0x0F;
            state.touch_x = (x_hi << 8) | x_lo;
            
            // Y: 12 bits
            let y_lo = (mid & 0xF0) >> 4;
            state.touch_y = (y_hi << 4) | y_lo;
        }
    }

    // Battery DualSense BT
    if data.len() >= 56 {
        let b_info = data[54];
        let b_level = b_info & 0x0F;
        state.battery = (b_level * 10).min(100);
        
        // In BT mode 0x31, charging status is usually found:
        // 1. In the upper nibble of byte 54 (0x1 - charging, 0x2 - full)
        // 2. In the lower nibble of byte 55 (0x1 - charging, 0x2 - full)
        let b_status = (b_info & 0xF0) >> 4;
        let power_status = data[55] & 0x0F;
        
        state.is_charging = b_status == 0x01 || b_status == 0x02 || 
                            power_status == 0x01 || power_status == 0x02;
    }

    state
}

// DS4 Parsing
pub fn parse_ds4(report: &[u8]) -> Option<GamepadState> {
    let report_id = report[0];

    // USB Report 0x01
    if report_id == 0x01 && report.len() >= 10 {
        return Some(parse_ds_common(&report[1..]));
    }

    // BT Report 0x11
    if report_id == 0x11 && report.len() >= 13 {
        // Input data starts at offset 3 usually (ID, something, something, Data)
        return Some(parse_ds_common(&report[3..]));
    }

    None
}

// Common structure for DS4 (mostly compatible layouts for buttons/sticks)
fn parse_ds_common(data: &[u8]) -> GamepadState {
    let mut state = GamepadState::default();
    if data.len() < 9 { return state; }

    // Sticks
    state.left_x = normalize_axis(data[0]);
    state.left_y = normalize_axis(data[1]);
    state.right_x = normalize_axis(data[2]);
    state.right_y = normalize_axis(data[3]);
    
    // Buttons Byte 4 (offset 4)
    let dpad = data[4] & 0x0F;
    match dpad {
        0 => state.dpad_up = true,
        1 => { state.dpad_up = true; state.dpad_right = true; },
        2 => state.dpad_right = true,
        3 => { state.dpad_right = true; state.dpad_down = true; },
        4 => state.dpad_down = true,
        5 => { state.dpad_down = true; state.dpad_left = true; },
        6 => state.dpad_left = true,
        7 => { state.dpad_left = true; state.dpad_up = true; },
        _ => {}
    }

    let buttons1 = data[4] >> 4;
    state.btn_square = (buttons1 & 0x1) != 0;
    state.btn_cross = (buttons1 & 0x2) != 0;
    state.btn_circle = (buttons1 & 0x4) != 0;
    state.btn_triangle = (buttons1 & 0x8) != 0;

    // Buttons Byte 5 (offset 5)
    let buttons2 = data[5];
    state.btn_l1 = (buttons2 & 0x01) != 0;
    state.btn_r1 = (buttons2 & 0x02) != 0;
    state.btn_share = (buttons2 & 0x10) != 0;
    state.btn_options = (buttons2 & 0x20) != 0;
    state.btn_l3 = (buttons2 & 0x40) != 0;
    state.btn_r3 = (buttons2 & 0x80) != 0;

    // PS Button is often in byte 6
    if data.len() >= 7 {
        state.btn_ps = (data[6] & 0x01) != 0;
        state.btn_touchpad = (data[6] & 0x02) != 0;
    }

    if data.len() >= 9 {
        state.l2 = normalize_trigger(data[7]);
        state.r2 = normalize_trigger(data[8]);
    }

    // Battery for DS4 USB (data[11])
    if data.len() >= 12 {
        let b_val = data[11];
        state.battery = ((b_val & 0x0F) * 10).min(100);
        state.is_charging = (b_val & 0x10) != 0;
    }

    state
}
