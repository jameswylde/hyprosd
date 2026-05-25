use std::fs;
use std::path::{Path, PathBuf};

pub fn find_backlight_device() -> Option<PathBuf> {
    let entries = fs::read_dir("/sys/class/backlight").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.join("brightness").exists() && path.join("max_brightness").exists() {
            return Some(path);
        }
    }
    None
}

pub fn read_brightness_percent(device: &Path) -> Option<u8> {
    read_brightness_percent_from_files(&device.join("brightness"), &device.join("max_brightness"))
}

pub fn read_brightness_percent_from_files(brightness_path: &Path, max_path: &Path) -> Option<u8> {
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

pub fn find_led(name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir("/sys/class/leds").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name()?.to_string_lossy().contains(name) && path.join("brightness").exists() {
            return Some(path.join("brightness"));
        }
    }
    None
}

pub fn read_led(path: &Path) -> Option<bool> {
    let value: u8 = fs::read_to_string(path).ok()?.trim().parse().ok()?;
    Some(value > 0)
}
