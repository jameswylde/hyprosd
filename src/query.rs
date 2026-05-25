use crate::sysfs;
use std::process::Command;

pub fn query_volume() -> Option<(u8, bool)> {
    // hyprland setups usually have wireplumber, but pactl keeps pulse users covered
    if let Some((level, muted)) = query_volume_wpctl() {
        return Some((level, muted));
    }
    query_volume_pactl()
}

pub fn query_brightness() -> Option<u8> {
    let device = sysfs::find_backlight_device()?;
    sysfs::read_brightness_percent(&device)
}

pub fn query_caps_lock() -> Option<bool> {
    sysfs::find_led("capslock").and_then(|path| sysfs::read_led(&path))
}

pub fn query_num_lock() -> Option<bool> {
    sysfs::find_led("numlock").and_then(|path| sysfs::read_led(&path))
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
