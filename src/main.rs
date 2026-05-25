mod backends;
mod config;
mod ipc;
mod query;
mod sysfs;
mod ui;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OsdEvent {
    Volume { level: u8, muted: bool },
    Brightness { level: u8 },
    CapsLock { on: bool },
    NumLock { on: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OsdCommand {
    Show { event: OsdEvent },
    // used when the cli cannot read the value directly but the daemon has seen it
    ShowCurrent { kind: OsdKind },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OsdKind {
    Volume,
    Brightness,
    CapsLock,
    NumLock,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Event(OsdEvent),
    State(OsdEvent),
    Command(OsdCommand),
}

#[derive(Parser)]
#[command(name = "hyprosd", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Daemon,
    Show {
        #[command(subcommand)]
        command: ShowCommand,
    },
}

#[derive(Subcommand)]
enum ShowCommand {
    Volume {
        level: Option<u8>,
        #[arg(long, default_value_t = false)]
        muted: bool,
    },
    Brightness {
        level: Option<u8>,
    },
    Caps {
        #[arg(value_enum)]
        state: Option<OnOff>,
    },
    Num {
        #[arg(value_enum)]
        state: Option<OnOff>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OnOff {
    On,
    Off,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Daemon => run_daemon(),
        Commands::Show { command } => send_show(command),
    }
}

fn send_show(show: ShowCommand) -> anyhow::Result<()> {
    match show {
        ShowCommand::Volume { level, muted } => {
            if let Some(level) = level {
                ipc::send_command(OsdCommand::Show {
                    event: OsdEvent::Volume { level, muted },
                })
            } else if let Some((level, current_muted)) = query::query_volume() {
                let muted = if muted { true } else { current_muted };
                ipc::send_command(OsdCommand::Show {
                    event: OsdEvent::Volume { level, muted },
                })
            } else {
                anyhow::bail!("unable to read volume level")
            }
        }
        ShowCommand::Brightness { level } => {
            if let Some(level) = level {
                ipc::send_command(OsdCommand::Show {
                    event: OsdEvent::Brightness { level },
                })
            } else if let Some(level) = query::query_brightness() {
                ipc::send_command(OsdCommand::Show {
                    event: OsdEvent::Brightness { level },
                })
            } else {
                anyhow::bail!("unable to read brightness level")
            }
        }
        ShowCommand::Caps { state } => match state {
            Some(OnOff::On) => ipc::send_command(OsdCommand::Show {
                event: OsdEvent::CapsLock { on: true },
            }),
            Some(OnOff::Off) => ipc::send_command(OsdCommand::Show {
                event: OsdEvent::CapsLock { on: false },
            }),
            None => {
                if let Some(on) = query::query_caps_lock() {
                    ipc::send_command(OsdCommand::Show {
                        event: OsdEvent::CapsLock { on },
                    })
                } else {
                    // some systems do not expose lock leds where the cli can read them
                    ipc::send_command(OsdCommand::ShowCurrent {
                        kind: OsdKind::CapsLock,
                    })
                }
            }
        },
        ShowCommand::Num { state } => match state {
            Some(OnOff::On) => ipc::send_command(OsdCommand::Show {
                event: OsdEvent::NumLock { on: true },
            }),
            Some(OnOff::Off) => ipc::send_command(OsdCommand::Show {
                event: OsdEvent::NumLock { on: false },
            }),
            None => {
                if let Some(on) = query::query_num_lock() {
                    ipc::send_command(OsdCommand::Show {
                        event: OsdEvent::NumLock { on },
                    })
                } else {
                    ipc::send_command(OsdCommand::ShowCurrent {
                        kind: OsdKind::NumLock,
                    })
                }
            }
        },
    }
}

fn run_daemon() -> anyhow::Result<()> {
    let config = Config::load_or_init().context("load config")?;

    gtk4::init().context("init GTK")?;
    let app = ui::OsdUi::new(&config).context("init OSD UI")?;

    let (sender, receiver) = async_channel::unbounded();

    // ipc handles explicit `hyprosd show ...` calls; backends notice changes on their own
    ipc::start_listener(sender.clone());
    backends::start(sender, config.backend.clone());

    let app = std::rc::Rc::new(std::cell::RefCell::new(app));
    let state = std::rc::Rc::new(std::cell::RefCell::new(OsdState::default()));
    let state_ref = state.clone();
    gtk4::glib::MainContext::default().spawn_local(async move {
        while let Ok(message) = receiver.recv().await {
            let mut state = state_ref.borrow_mut();
            let event = match message {
                AppMessage::Event(event) => event,
                AppMessage::State(event) => {
                    update_state(&mut state, &event);
                    continue;
                }
                AppMessage::Command(command) => match command_to_event(&mut state, command) {
                    Some(event) => event,
                    None => continue,
                },
            };
            // keep the last values around so `show current` has something to display
            update_state(&mut state, &event);
            drop(state);
            app.borrow_mut().show_event(event);
        }
    });

    gtk4::glib::MainLoop::new(None, false).run();
    Ok(())
}

#[derive(Default)]
struct OsdState {
    volume: Option<(u8, bool)>,
    brightness: Option<u8>,
    caps: Option<bool>,
    num: Option<bool>,
}

fn update_state(state: &mut OsdState, event: &OsdEvent) {
    match event {
        OsdEvent::Volume { level, muted } => state.volume = Some((*level, *muted)),
        OsdEvent::Brightness { level } => state.brightness = Some(*level),
        OsdEvent::CapsLock { on } => state.caps = Some(*on),
        OsdEvent::NumLock { on } => state.num = Some(*on),
    }
}

fn command_to_event(state: &mut OsdState, command: OsdCommand) -> Option<OsdEvent> {
    match command {
        OsdCommand::Show { event } => Some(event),
        OsdCommand::ShowCurrent { kind } => match kind {
            OsdKind::Volume => state
                .volume
                .map(|(level, muted)| OsdEvent::Volume { level, muted }),
            OsdKind::Brightness => state.brightness.map(|level| OsdEvent::Brightness { level }),
            OsdKind::CapsLock => state.caps.map(|on| OsdEvent::CapsLock { on }),
            OsdKind::NumLock => state.num.map(|on| OsdEvent::NumLock { on }),
        },
    }
}
