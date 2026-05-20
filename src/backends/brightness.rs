use crate::{AppMessage, OsdEvent};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

pub fn start(sender: async_channel::Sender<AppMessage>, configured_path: Option<PathBuf>) {
    thread::spawn(move || {
        if let Err(err) = watch_brightness(sender, configured_path) {
            eprintln!("hyprosd brightness watcher error: {err:?}");
        }
    });
}

fn watch_brightness(
    sender: async_channel::Sender<AppMessage>,
    configured_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    let device = configured_path.or_else(find_backlight_device);
    let Some(device) = device else {
        eprintln!("hyprosd: no backlight device found");
        return Ok(());
    };

    let brightness_path = device.join("brightness");
    let max_path = device.join("max_brightness");
    let brightness_cb = brightness_path.clone();
    let max_cb = max_path.clone();

    let sender_cb = sender.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res
                && matches!(event.kind, EventKind::Modify(_))
                && let Some(level) = read_brightness_percent(&brightness_cb, &max_cb)
            {
                let _ = sender_cb.send_blocking(AppMessage::Event(OsdEvent::Brightness { level }));
            }
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(250)),
    )?;

    // brightness changes are already reflected in sysfs, so watching the file is cheap
    watcher.watch(&brightness_path, RecursiveMode::NonRecursive)?;

    // send the initial value so `show current` has state before the first change
    if let Some(level) = read_brightness_percent(&brightness_path, &max_path) {
        let _ = sender.send_blocking(AppMessage::State(OsdEvent::Brightness { level }));
    }

    loop {
        thread::park();
    }
}

fn find_backlight_device() -> Option<PathBuf> {
    let entries = fs::read_dir("/sys/class/backlight").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.join("brightness").exists() && path.join("max_brightness").exists() {
            return Some(path);
        }
    }
    None
}

fn read_brightness_percent(brightness_path: &Path, max_path: &Path) -> Option<u8> {
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
