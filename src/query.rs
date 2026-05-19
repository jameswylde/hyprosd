use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn query_volume() -> Option<(u8, bool)> {
    // hyprland setups usually have wireplumber, but pactl keeps pulse users covered
    if let Some((level, muted)) = query_volume_wpctl() {
        return Some((level, muted));
    }
    query_volume_pactl()
}

pub fn query_brightness() -> Option<u8> {
    let device = find_backlight_device()?;
    read_brightness_percent(&device.join("brightness"), &device.join("max_brightness"))
}

pub fn query_caps_lock() -> Option<bool> {
    find_led("capslock").and_then(read_led)
}

pub fn query_num_lock() -> Option<bool> {
    find_led("numlock").and_then(read_led)
}

fn query_volume_wpctl() -> Option<(u8, bool)> {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let muted = stdout.contains("[MUTED]");
    let value = stdout
        .split_whitespace()
        .find_map(|token| token.trim_end_matches(',').parse::<f32>().ok())?;
    let percent = (value * 100.0).round() as i32;
    Some((percent.clamp(0, 100) as u8, muted))
}

fn query_volume_pactl() -> Option<(u8, bool)> {
    let volume_out = Command::new("pactl")
        .args(["get-sink-volume", "@DEFAULT_SINK@"])
        .output()
        .ok()?;
    if !volume_out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&volume_out.stdout);
    let percent = stdout
        .split_whitespace()
        .find_map(|token| token.strip_suffix('%'))
        .and_then(|value| value.parse::<u8>().ok())?;

    let muted_out = Command::new("pactl")
        .args(["get-sink-mute", "@DEFAULT_SINK@"])
        .output()
        .ok()?;
    let muted_stdout = String::from_utf8_lossy(&muted_out.stdout);
    let muted = muted_stdout.to_ascii_lowercase().contains("yes");

    Some((percent.min(100), muted))
}

fn find_backlight_device() -> Option<PathBuf> {
    // use the first real backlight device; custom selection can come from config later
    let entries = fs::read_dir("/sys/class/backlight").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.join("brightness").exists() && path.join("max_brightness").exists() {
            return Some(path);
        }
    }
    None
}

fn read_brightness_percent(brightness_path: &PathBuf, max_path: &PathBuf) -> Option<u8> {
    let brightness: u32 = fs::read_to_string(brightness_path)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    let max: u32 = fs::read_to_string(max_path).ok()?.trim().parse().ok()?;
    if max == 0 {
        return None;
    }
    let percent = ((brightness as f64 / max as f64) * 100.0).round() as u8;
    Some(percent.min(100))
}

fn find_led(name: &str) -> Option<PathBuf> {
    // kernel led names vary, so match the useful part rather than a full device name
    let entries = fs::read_dir("/sys/class/leds").ok()?;
    for entry in entries.flatten() {
        let path = entry.path().join("brightness");
        if path.exists() && path.to_string_lossy().contains(name) {
            return Some(path);
        }
    }
    None
}

fn read_led(path: PathBuf) -> Option<bool> {
    let value: u8 = fs::read_to_string(path).ok()?.trim().parse().ok()?;
    Some(value > 0)
}
