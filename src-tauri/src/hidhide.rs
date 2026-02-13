use std::process::Command;
use std::os::windows::process::CommandExt;

const HIDHIDE_CLI_PATH: &str = r"C:\Program Files\Nefarius Software Solutions\HidHide\x64\HidHideCLI.exe";
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn is_installed() -> bool {
    std::path::Path::new(HIDHIDE_CLI_PATH).exists()
}

pub fn whitelist_self() -> anyhow::Result<()> {
    let current_exe = std::env::current_exe()?;
    let path_str = current_exe.to_str().unwrap_or_default();
    
    run_hidhide(&["--app-reg", path_str])
}

#[allow(dead_code)]
pub fn unwhitelist_self() -> anyhow::Result<()> {
    let current_exe = std::env::current_exe()?;
    let path_str = current_exe.to_str().unwrap_or_default();
    
    run_hidhide(&["--app-unreg", path_str])
}

pub fn hide_device(instance_id: &str) -> anyhow::Result<()> {
    run_hidhide(&["--dev-hide", instance_id])?;
    // Ensure global cloak is on, otherwise individual hiding doesn't work
    run_hidhide(&["--cloak-on"])
}

pub fn unhide_device(instance_id: &str) -> anyhow::Result<()> {
    run_hidhide(&["--dev-unhide", instance_id])
}

fn run_hidhide(args: &[&str]) -> anyhow::Result<()> {
    if !is_installed() {
        return Err(anyhow::anyhow!("HidHideCLI not found"));
    }

    // log::info!("Executing HidHideCLI with args: {:?}", args);

    let output = Command::new(HIDHIDE_CLI_PATH)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        log::error!("HidHideCLI error. Args: {:?}. Status: {}. Stderr: {}. Stdout: {}", args, output.status, err, out);
        return Err(anyhow::anyhow!("HidHide error: {} (Details in logs)", err.trim()));
    }

    Ok(())
}

/// Converts a Windows Device Path (from hidapi) to an Instance ID
pub fn path_to_instance_id(path: &str) -> Option<String> {
    // Find "HID#" anchor case-insensitively
    let upper = path.to_uppercase();
    let start_idx = upper.find("HID#")?;
    
    // Use the original string slice starting from "HID#"
    let useful_part = &path[start_idx..];
    
    let parts: Vec<&str> = useful_part.split('#').collect();
    
    if parts.len() < 3 {
        return None;
    }

    // parts[0] -> "HID"
    // parts[1] -> Hardware ID
    // parts[2] -> Instance ID
    
    let hardware_id = parts[1].to_uppercase(); 
    let instance_id = parts[2].to_uppercase();

    Some(format!(r"HID\{}\{}", hardware_id, instance_id))
}
