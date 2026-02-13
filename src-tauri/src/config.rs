use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use crate::mapping::{ButtonMapping, PhysicalButton, MappingTarget};

pub const APP_NAME: &str = "DX3";

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub mappings: Vec<ButtonMapping>,
    #[serde(default = "default_deadzone")]
    pub deadzone_left: f32,
    #[serde(default = "default_deadzone")]
    pub deadzone_right: f32,
    #[serde(default = "default_mouse_sens")]
    pub mouse_sens_left: f32,
    #[serde(default = "default_mouse_sens")]
    pub mouse_sens_right: f32,
    #[serde(default = "default_mouse_sens")]
    pub mouse_sens_touchpad: f32,
    #[serde(default = "default_rgb_r")]
    pub rgb_r: u8,
    #[serde(default = "default_rgb_g")]
    pub rgb_g: u8,
    #[serde(default = "default_rgb_b")]
    pub rgb_b: u8,
    #[serde(default = "default_rgb_bright")]
    pub rgb_brightness: u8,
    #[serde(default)]
    pub show_battery_led: bool,
    #[serde(default)]
    pub trigger_l2_mode: u8,
    #[serde(default)]
    pub trigger_l2_start: u8,
    #[serde(default)]
    pub trigger_l2_force: u8,
    #[serde(default)]
    pub trigger_r2_mode: u8,
    #[serde(default)]
    pub trigger_r2_start: u8,
    #[serde(default)]
    pub trigger_r2_force: u8,
    #[serde(default)]
    pub player_led_brightness: u8,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            mappings: AppConfig::default_mappings(),
            deadzone_left: 0.1,
            deadzone_right: 0.1,
            mouse_sens_left: 25.0,
            mouse_sens_right: 25.0,
            mouse_sens_touchpad: 25.0,
            rgb_r: 0,
            rgb_g: 0,
            rgb_b: 255,
            rgb_brightness: 255,
            show_battery_led: false,
            trigger_l2_mode: 0,
            trigger_l2_start: 0,
            trigger_l2_force: 0,
            trigger_r2_mode: 0,
            trigger_r2_start: 0,
            trigger_r2_force: 0,
            player_led_brightness: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub hide_controller: bool,
    #[serde(default)]
    pub start_minimized: bool,
    pub mappings: Vec<ButtonMapping>,
    #[serde(default = "default_deadzone")]
    pub deadzone_left: f32,
    #[serde(default = "default_deadzone")]
    pub deadzone_right: f32,
    #[serde(default = "default_mouse_sens")]
    pub mouse_sens_left: f32,
    #[serde(default = "default_mouse_sens")]
    pub mouse_sens_right: f32,
    #[serde(default = "default_mouse_sens")]
    pub mouse_sens_touchpad: f32,
    #[serde(default)]
    pub active_profile: String,
    #[serde(default = "default_rgb_r")]
    pub rgb_r: u8,
    #[serde(default = "default_rgb_g")]
    pub rgb_g: u8,
    #[serde(default = "default_rgb_b")]
    pub rgb_b: u8,
    #[serde(default = "default_rgb_bright")]
    pub rgb_brightness: u8,
    #[serde(default)]
    pub show_battery_led: bool,
    // Adaptive Triggers
    #[serde(default)]
    pub trigger_l2_mode: u8,
    #[serde(default)]
    pub trigger_l2_start: u8,
    #[serde(default)]
    pub trigger_l2_force: u8,
    #[serde(default)]
    pub trigger_r2_mode: u8,
    #[serde(default)]
    pub trigger_r2_start: u8,
    #[serde(default)]
    pub trigger_r2_force: u8,
    #[serde(default)]
    pub player_led_brightness: u8, // 0=High, 1=Med, 2=Low
}

fn default_deadzone() -> f32 { 0.1 }
fn default_mouse_sens() -> f32 { 25.0 }
fn default_rgb_r() -> u8 { 0 }
fn default_rgb_g() -> u8 { 0 }
fn default_rgb_b() -> u8 { 255 }
fn default_rgb_bright() -> u8 { 255 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hide_controller: true,
            start_minimized: false,
            mappings: Self::default_mappings(),
            deadzone_left: 0.1,
            deadzone_right: 0.1,
            mouse_sens_left: 25.0,
            mouse_sens_right: 25.0,
            mouse_sens_touchpad: 25.0,
            active_profile: "Default".to_string(),
            rgb_r: 0,
            rgb_g: 0,
            rgb_b: 255,
            rgb_brightness: 255,
            show_battery_led: false,
            trigger_l2_mode: 0,
            trigger_l2_start: 0,
            trigger_l2_force: 0,
            trigger_r2_mode: 0,
            trigger_r2_start: 0,
            trigger_r2_force: 0,
            player_led_brightness: 0,
        }
    }
}

impl AppConfig {
    pub fn save_internal(
        hide: bool, min: bool, mappings: Vec<ButtonMapping>, 
        dl: f32, dr: f32, p: String, msl: f32, msr: f32, mst: f32,
        r: u8, g: u8, b: u8, bright: u8, bat_led: bool,
        tl2_mode: u8, tl2_start: u8, tl2_force: u8,
        tr2_mode: u8, tr2_start: u8, tr2_force: u8,
        pled_bright: u8,
    ) {
        let config = AppConfig { 
            hide_controller: hide,
            start_minimized: min,
            mappings: mappings.clone(),
            deadzone_left: dl,
            deadzone_right: dr,
            mouse_sens_left: msl,
            mouse_sens_right: msr,
            mouse_sens_touchpad: mst,
            active_profile: p.clone(),
            rgb_r: r,
            rgb_g: g,
            rgb_b: b,
            rgb_brightness: bright,
            show_battery_led: bat_led,
            trigger_l2_mode: tl2_mode,
            trigger_l2_start: tl2_start,
            trigger_l2_force: tl2_force,
            trigger_r2_mode: tr2_mode,
            trigger_r2_start: tr2_start,
            trigger_r2_force: tr2_force,
            player_led_brightness: pled_bright,
        };
        config.save();
    }
    pub fn default_mappings() -> Vec<ButtonMapping> {
        vec![
            ButtonMapping { source: PhysicalButton::Cross, targets: vec![MappingTarget::Xbox(0x1000)] },    // A
            ButtonMapping { source: PhysicalButton::Circle, targets: vec![MappingTarget::Xbox(0x2000)] },   // B
            ButtonMapping { source: PhysicalButton::Square, targets: vec![MappingTarget::Xbox(0x4000)] },   // X
            ButtonMapping { source: PhysicalButton::Triangle, targets: vec![MappingTarget::Xbox(0x8000)] }, // Y
            ButtonMapping { source: PhysicalButton::L1, targets: vec![MappingTarget::Xbox(0x0100)] },       // LB
            ButtonMapping { source: PhysicalButton::R1, targets: vec![MappingTarget::Xbox(0x0200)] },       // RB
            ButtonMapping { source: PhysicalButton::L3, targets: vec![MappingTarget::Xbox(0x0040)] },       // LThumb
            ButtonMapping { source: PhysicalButton::R3, targets: vec![MappingTarget::Xbox(0x0080)] },       // RThumb
            ButtonMapping { source: PhysicalButton::Options, targets: vec![MappingTarget::Xbox(0x0010)] },  // Start
            ButtonMapping { source: PhysicalButton::Share, targets: vec![MappingTarget::Xbox(0x0020)] },    // Back
            ButtonMapping { source: PhysicalButton::PS, targets: vec![MappingTarget::Xbox(0x0400)] },       // Guide
            ButtonMapping { source: PhysicalButton::DpadUp, targets: vec![MappingTarget::Xbox(0x0001)] },
            ButtonMapping { source: PhysicalButton::DpadDown, targets: vec![MappingTarget::Xbox(0x0002)] },
            ButtonMapping { source: PhysicalButton::DpadLeft, targets: vec![MappingTarget::Xbox(0x0004)] },
            ButtonMapping { source: PhysicalButton::DpadRight, targets: vec![MappingTarget::Xbox(0x0008)] },
            ButtonMapping { source: PhysicalButton::LeftStick, targets: vec![MappingTarget::XboxLS] },
            ButtonMapping { source: PhysicalButton::RightStick, targets: vec![MappingTarget::XboxRS] },
            ButtonMapping { source: PhysicalButton::L2, targets: vec![MappingTarget::XboxLT] },
            ButtonMapping { source: PhysicalButton::R2, targets: vec![MappingTarget::XboxRT] },
            ButtonMapping { source: PhysicalButton::Touchpad, targets: vec![] },
            ButtonMapping { source: PhysicalButton::TouchpadLeft, targets: vec![] },
            ButtonMapping { source: PhysicalButton::TouchpadRight, targets: vec![] },
            ButtonMapping { source: PhysicalButton::Mute, targets: vec![] },
        ]
    }

    pub fn config_path() -> PathBuf {
        let mut path = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        path.push(APP_NAME);
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        path.push("config.json");
        path
    }

    pub fn load() -> Self {
        fs::read_to_string(Self::config_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = fs::write(Self::config_path(), s);
        }
    }

    pub fn profiles_dir() -> PathBuf {
        let mut path = Self::config_path().parent().unwrap().to_path_buf();
        path.push("profiles");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        path
    }

    pub fn list_profiles() -> Vec<String> {
        let dir = Self::profiles_dir();
        let mut profiles = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("json") {
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                profiles.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        profiles
    }

    pub fn save_profile(name: &str, profile: &Profile) {
        let mut path = Self::profiles_dir();
        path.push(format!("{}.json", name));
        if let Ok(s) = serde_json::to_string_pretty(profile) {
            let _ = fs::write(path, s);
        }
    }

    pub fn load_profile(name: &str) -> Option<Profile> {
        let mut path = Self::profiles_dir();
        path.push(format!("{}.json", name));
        let content = fs::read_to_string(path).ok()?;
        
        // 1. Try parsing as new Profile struct
        if let Ok(p) = serde_json::from_str::<Profile>(&content) {
            return Some(p);
        }
        
        // 2. Fallback: Legacy Vec<ButtonMapping>
        if let Ok(mappings) = serde_json::from_str::<Vec<ButtonMapping>>(&content) {
            return Some(Profile {
                mappings,
                ..Default::default()
            });
        }
        
        None
    }

    pub fn delete_profile(name: &str) {
        let mut path = Self::profiles_dir();
        path.push(format!("{}.json", name));
        let _ = fs::remove_file(path);
    }
}
