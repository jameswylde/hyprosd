use crate::{AppMessage, OsdCommand};
use anyhow::Context;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::thread;

pub fn send_command(command: OsdCommand) -> anyhow::Result<()> {
    let path = socket_path()?;
    let mut stream = UnixStream::connect(&path).context("connect to daemon socket")?;
    let payload = serde_json::to_vec(&command).context("serialize command")?;
    stream.write_all(&payload).context("send command")?;
    Ok(())
}

pub fn start_listener(sender: async_channel::Sender<AppMessage>) {
    thread::spawn(move || {
        if let Err(err) = listen_loop(sender) {
            eprintln!("hyprosd ipc listener error: {err:?}");
        }
    });
}

fn listen_loop(sender: async_channel::Sender<AppMessage>) -> anyhow::Result<()> {
    let path = socket_path()?;
    if path.exists() {
        if UnixStream::connect(&path).is_ok() {
            anyhow::bail!("daemon already running");
        }
        // if connect fails, the socket is from a dead daemon and can be replaced
        std::fs::remove_file(&path).context("remove stale socket")?;
    }
    let listener = UnixListener::bind(&path).context("bind socket")?;
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(err) = handle_client(&mut stream, &sender) {
                    eprintln!("hyprosd ipc client error: {err:?}");
                }
            }
            Err(err) => eprintln!("hyprosd ipc accept error: {err:?}"),
        }
    }
    Ok(())
}

fn handle_client(
    stream: &mut UnixStream,
    sender: &async_channel::Sender<AppMessage>,
) -> anyhow::Result<()> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).context("read command")?;
    let command: OsdCommand = serde_json::from_slice(&buf).context("decode command")?;
    let _ = sender.send_blocking(AppMessage::Command(command));
    Ok(())
}

fn socket_path() -> anyhow::Result<PathBuf> {
    // runtime dir keeps the socket per-user and cleaned up with the login session
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .context("XDG_RUNTIME_DIR not set")?;
    Ok(runtime.join("hyprosd.sock"))
}
