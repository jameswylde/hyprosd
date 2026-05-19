use crate::{AppMessage, OsdEvent};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

pub fn start(sender: async_channel::Sender<AppMessage>) {
    thread::spawn(move || {
        if let Err(err) = watch_locks(sender) {
            eprintln!("hyprosd lock watcher error: {err:?}");
        }
    });
}

fn watch_locks(sender: async_channel::Sender<AppMessage>) -> anyhow::Result<()> {
    let caps = find_led("capslock");
    let num = find_led("numlock");

    if caps.is_none() && num.is_none() {
        eprintln!("hyprosd: no caps/num lock LEDs found");
        return Ok(());
    }

    let caps_for_cb = caps.clone();
    let num_for_cb = num.clone();
    let sender_cb = sender.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res
                && matches!(event.kind, EventKind::Modify(_))
            {
                if let Some(path) = caps_for_cb.as_deref()
                    && let Some(on) = read_led(path)
                {
                    let _ = sender_cb.send_blocking(AppMessage::Event(OsdEvent::CapsLock { on }));
                }
                if let Some(path) = num_for_cb.as_deref()
                    && let Some(on) = read_led(path)
                {
                    let _ = sender_cb.send_blocking(AppMessage::Event(OsdEvent::NumLock { on }));
                }
            }
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(250)),
    )?;

    if let Some(path) = caps.as_deref() {
        watcher.watch(path, RecursiveMode::NonRecursive)?;
    }
    if let Some(path) = num.as_deref() {
        watcher.watch(path, RecursiveMode::NonRecursive)?;
    }

    // seed the cache so manual `show caps` and `show num` can display immediately
    if let Some(path) = caps.as_deref()
        && let Some(on) = read_led(path)
    {
        let _ = sender.send_blocking(AppMessage::Event(OsdEvent::CapsLock { on }));
    }
    if let Some(path) = num.as_deref()
        && let Some(on) = read_led(path)
    {
        let _ = sender.send_blocking(AppMessage::Event(OsdEvent::NumLock { on }));
    }

    let mut last_caps = caps.as_deref().and_then(read_led);
    let mut last_num = num.as_deref().and_then(read_led);
    loop {
        thread::sleep(Duration::from_millis(200));
        // notify can miss led writes on some setups, so the poller is the backup path
        if let Some(path) = caps.as_deref()
            && let Some(on) = read_led(path)
            && last_caps != Some(on)
        {
            last_caps = Some(on);
            let _ = sender.send_blocking(AppMessage::Event(OsdEvent::CapsLock { on }));
        }
        if let Some(path) = num.as_deref()
            && let Some(on) = read_led(path)
            && last_num != Some(on)
        {
            last_num = Some(on);
            let _ = sender.send_blocking(AppMessage::Event(OsdEvent::NumLock { on }));
        }
    }
}

fn find_led(name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir("/sys/class/leds").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name()?.to_string_lossy().contains(name) && path.join("brightness").exists() {
            return Some(path.join("brightness"));
        }
    }
    None
}

fn read_led(path: &Path) -> Option<bool> {
    let value: u8 = fs::read_to_string(path).ok()?.trim().parse().ok()?;
    Some(value > 0)
}
