use std::process::Command;
use regex::Regex;

// Helper function to run commands with a timeout
fn run_command_with_timeout(command: &str, args: &[&str]) -> Option<String> {
    // Add a timeout of 1 second for external commands to prevent hanging
    let mut full_args = vec!["1"];
    full_args.push(command);
    full_args.extend_from_slice(args);

    if let Ok(output) = Command::new("timeout")
        .args(&full_args)
        .output() {
        if output.status.success() {
            String::from_utf8(output.stdout).ok()
        } else {
            // Log stderr if command failed
            eprintln!(
                "Command `{}` with args `{:?}` failed: {}",
                command,
                args,
                String::from_utf8_lossy(&output.stderr)
            );
            None
        }
    } else {
        eprintln!("Failed to execute command: {}", command);
        None
    }
}

pub fn get_volume() -> Option<f32> {
    let output_str = run_command_with_timeout("pactl", &["get-sink-volume", "@DEFAULT_SINK@"])?;
    let re = Regex::new(r"(\d+)%").unwrap();
    let caps = re.captures(&output_str)?;
    let value_str = caps.get(1)?.as_str();
    value_str.parse::<f32>().ok()
}

pub fn set_volume_cmd(value: u8) {
    let _ = Command::new("pactl")
        .arg("set-sink-volume")
        .arg("@DEFAULT_SINK@")
        .arg(format!("{}%", value))
        .output();
}

pub fn get_brightness() -> Option<f32> {
    let current_str = run_command_with_timeout("brightnessctl", &["g"])?;
    let current = current_str.trim().parse::<f32>().ok()?;

    let max_str = run_command_with_timeout("brightnessctl", &["m"])?;
    let max = max_str.trim().parse::<f32>().ok()?;

    if max > 0.0 {
        Some((current / max) * 100.0)
    } else {
        None
    }
}

pub fn set_brightness_cmd(value: u8) {
    let _ = Command::new("brightnessctl")
        .arg("s")
        .arg(format!("{}%", value))
        .output();
}

pub fn fetch_wifi_status() -> (bool, String) {
    // Try nmcli first (NetworkManager)
    if let Some(stdout) = run_command_with_timeout("nmcli", &["-t", "-f", "ACTIVE,SSID", "dev", "wifi"]) {
        for line in stdout.lines() {
            if line.starts_with("yes:") {
                let ssid = line.strip_prefix("yes:").unwrap_or("Connected");
                return (true, ssid.to_string());
            }
        }
    }

    // Fallback to iwgetid
    if let Some(ssid) = run_command_with_timeout("iwgetid", &["-r"]) {
        let ssid = ssid.trim();
        if !ssid.is_empty() {
            return (true, ssid.to_string());
        }
    }

    // Check if WiFi is disabled via nmcli
    if let Some(stdout) = run_command_with_timeout("nmcli", &["radio", "wifi"]) {
        if stdout.trim() == "disabled" {
            return (false, "WiFi Off".to_string());
        }
    }

    // WiFi is on but not connected or other issue
    (true, "No Network".to_string())
}

pub fn toggle_wifi_cmd(enable: bool) {
    if enable {
        let _ = Command::new("nmcli").args(&["radio", "wifi", "on"]).output();
    } else {
        let _ = Command::new("nmcli").args(&["radio", "wifi", "off"]).output();
    }
}

pub fn fetch_bluetooth_status() -> (bool, String) {
    // Check if bluetooth is powered on using bluetoothctl
    if let Some(stdout) = run_command_with_timeout("bluetoothctl", &["show"]) {
        let powered = stdout.lines()
            .find(|line| line.contains("Powered:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim() == "yes")
            .unwrap_or(false);

        if !powered {
            return (false, "Bluetooth Off".to_string());
        }

        // Check for connected devices
        if let Some(devices_str) = run_command_with_timeout("bluetoothctl", &["devices", "Connected"]) {
            if let Some(first_device) = devices_str.lines().next() {
                // Extract device name (format: "Device MAC_ADDRESS Name")
                let parts: Vec<&str> = first_device.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[2..].join(" ");
                    return (true, name);
                }
            }
        }

        // Bluetooth is on but no device connected
        return (true, "No Device".to_string());
    }

    // Fallback - assume bluetooth is available but off
    (false, "Bluetooth Off".to_string())
}

pub fn toggle_bluetooth_cmd(enable: bool) {
    if enable {
        let _ = Command::new("bluetoothctl").args(&["power", "on"]).output();
    } else {
        let _ = Command::new("bluetoothctl").args(&["power", "off"]).output();
    }
}

pub fn toggle_eye_care_cmd(enable: bool) {
    if enable {
        let _ = Command::new("redshift").args(&["-P", "-O", "3500"]).output();
    } else {
        let _ = Command::new("redshift").args(&["-x"]).output();
    }
}
