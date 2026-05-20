use crate::{AppMessage, OsdEvent, query};
use std::thread;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_millis(200);

pub fn start(sender: async_channel::Sender<AppMessage>) {
    thread::spawn(move || watch_volume(sender));
}

fn watch_volume(sender: async_channel::Sender<AppMessage>) {
    let mut last = query::query_volume();

    if let Some((level, muted)) = last {
        let _ = sender.send_blocking(AppMessage::State(OsdEvent::Volume { level, muted }));
    } else {
        eprintln!("hyprosd: unable to read volume level");
    }

    loop {
        thread::sleep(POLL_INTERVAL);

        // there is no simple portable file event for default sink volume, so polling wins here
        let current = query::query_volume();
        if current != last {
            last = current;
            if let Some((level, muted)) = current {
                let _ = sender.send_blocking(AppMessage::Event(OsdEvent::Volume { level, muted }));
            }
        }
    }
}
