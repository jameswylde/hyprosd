use crate::AppMessage;
use crate::config::BackendConfig;

mod brightness;
mod locks;
mod volume;

pub fn start(sender: async_channel::Sender<AppMessage>, config: BackendConfig) {
    // each backend owns its own thread so slow system reads do not block the gtk loop
    brightness::start(sender.clone(), config.brightness_path);
    locks::start(sender.clone());
    volume::start(sender);
}
