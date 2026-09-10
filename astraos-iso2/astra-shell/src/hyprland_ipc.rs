// hyprland_ipc.rs - Hyprland Wayland compositor IPC client.
//
// Hyprland exposes two UNIX domain sockets per session:
//   * `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
//       - command socket (send a command, receive a reply).
//   * `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock`
//       - event socket (server pushes one event per line, forever).
//
// This module only listens on `.socket2.sock` and parses the small set
// of events the bar / dock care about: `workspace>>N` and
// `activewindow>>class,title`. The listener runs on a dedicated OS
// thread that owns a private Tokio runtime, so the GTK main thread is
// never blocked and `main.rs` does not need to install a runtime
// context.
//
// Future work (task 2-b): wire parsed events into a
// `tokio::sync::broadcast` channel so the bar / dock can subscribe to
// workspace + window changes and update themselves reactively instead
// of polling.

use log::{error, info};
use std::env;
use std::path::PathBuf;
use std::thread;
use tokio::net::UnixStream;
use tokio::runtime::Runtime;

/// A high-level Hyprland event emitted by the compositor's
/// `.socket2.sock`.
///
/// NOTE: variant payload fields are consumed by `Debug` formatting
/// today and will be read by the bar / dock subscribers in task 2-b -
/// that's why we silence the dead-code warning below.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum HyprlandEvent {
    /// User switched to workspace `id`.
    WorkspaceChanged(u32),
    /// Focused window changed - payload is `"class,title"`
    /// (Hyprland format).
    ActiveWindowChanged(String),
    /// A window was opened - payload is `"class,title"`.
    WindowOpened(String),
    /// A window was closed - payload is the Hyprland window address.
    WindowClosed(String),
}

/// Connection holder for the Hyprland IPC sockets.
///
/// `new()` discovers the socket path from the environment;
/// `connect_signals()` spawns a dedicated OS thread that hosts its own
/// Tokio runtime and runs the async listener for the lifetime of the
/// shell process.
pub struct HyprlandClient {
    socket_path: Option<PathBuf>,
}

impl HyprlandClient {
    /// Build a new client. The socket path is resolved from
    /// `$HYPRLAND_INSTANCE_SIGNATURE` + `$XDG_RUNTIME_DIR`.
    ///
    /// Returns a client with `socket_path = None` when not running
    /// under Hyprland - `connect_signals()` will then log an error and
    /// do nothing, leaving the shell running in a degraded state.
    pub fn new() -> Self {
        Self {
            socket_path: Self::find_event_socket(),
        }
    }

    /// Resolve the event socket (`.socket2.sock`) path.
    ///
    /// Format:
    /// `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock`
    ///
    /// Returns `None` when either environment variable is unset - the
    /// shell then runs without Hyprland event integration (e.g. when
    /// launched outside a Hyprland session, like in a dev container).
    fn find_event_socket() -> Option<PathBuf> {
        let instance_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
        let runtime_dir = env::var("XDG_RUNTIME_DIR").ok()?;
        Some(
            PathBuf::from(runtime_dir)
                .join("hypr")
                .join(&instance_sig)
                .join(".socket2.sock"),
        )
    }

    /// Spawn the async event listener on a dedicated OS thread that
    /// owns its own Tokio runtime.
    ///
    /// If the socket can't be found (we're not running under Hyprland),
    /// this is a no-op with an error log - the shell still starts so the
    /// user can launch apps manually from the launcher.
    ///
    /// The dedicated thread + private runtime design keeps the GTK main
    /// thread free from async concerns: `main.rs` doesn't need to
    /// install a Tokio runtime context, and the listener can never
    /// block GTK redraws.
    pub fn connect_signals(&self) {
        let Some(path) = self.socket_path.as_ref().cloned() else {
            error!(
                "HYPRLAND_INSTANCE_SIGNATURE or XDG_RUNTIME_DIR not set - \
                 Hyprland IPC disabled. Run astra-shell under Hyprland to \
                 enable workspace/window events."
            );
            return;
        };

        info!("Connecting to Hyprland event socket: {:?}", path);

        // Spawn the listener thread. It owns the socket path + a private
        // Tokio runtime, and runs forever (until the process exits).
        thread::Builder::new()
            .name("hyprland-ipc".to_string())
            .spawn(move || {
                let rt = match Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        error!("Failed to create Tokio runtime: {}", e);
                        return;
                    }
                };
                rt.block_on(async move {
                    // Reconnect loop - Hyprland may restart while the
                    // shell is running. The 2 s back-off prevents a hot
                    // loop on a missing socket.
                    loop {
                        match UnixStream::connect(&path).await {
                            Ok(stream) => {
                                info!("Connected to Hyprland IPC");
                                if let Err(e) = listen_events(stream).await {
                                    error!("Hyprland listener error: {}", e);
                                }
                                info!("Hyprland IPC stream closed, reconnecting in 2s...");
                            }
                            Err(e) => {
                                error!("Failed to connect to Hyprland socket {:?}: {}", path, e);
                            }
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                });
            })
            .expect("failed to spawn hyprland-ipc thread");
    }
}

impl Default for HyprlandClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Async event-reader loop. Reads one event per line from the socket,
/// parses each line into a [`HyprlandEvent`], and logs it. Returns when
/// the socket is closed or an unrecoverable read error occurs.
///
/// The error type is `std::io::Error` (not `Box<dyn Error>`) so the
/// resulting future is `Send` and can run on a multi-thread runtime.
async fn listen_events(mut stream: UnixStream) -> std::io::Result<()> {
    use tokio::io::AsyncBufReadExt;

    let mut reader = tokio::io::BufReader::new(&mut stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF - compositor closed the socket.
            Ok(_) => {
                if let Some(ev) = parse_event(&line) {
                    info!("Hyprland event: {:?}", ev);
                    // TODO task 2-b: broadcast `ev` on a
                    // tokio::sync::broadcast channel so the bar / dock
                    // can update themselves.
                }
            }
            Err(e) => {
                error!("Error reading from Hyprland socket: {}", e);
                break;
            }
        }
    }
    Ok(())
}

/// Parse one Hyprland event line.
///
/// Format (from Hyprland's `socket2` docs):
///   `workspace>>2`            - workspace changed to ID 2
///   `activewindow>>kitty,top` - focused window is `kitty` titled `top`
///   `openwindow>>addr,ws,title,class` - full open-window event
///   `closewindow>>addr`       - window close event
///
/// We only model the subset the bar / dock need today; unknown events
/// are silently dropped (Hyprland emits dozens of event types - we'll
/// wire up more as needed).
fn parse_event(line: &str) -> Option<HyprlandEvent> {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("workspace>>") {
        if let Ok(id) = rest.parse::<u32>() {
            return Some(HyprlandEvent::WorkspaceChanged(id));
        }
    }
    if let Some(rest) = line.strip_prefix("activewindow>>") {
        return Some(HyprlandEvent::ActiveWindowChanged(rest.to_string()));
    }
    if let Some(rest) = line.strip_prefix("openwindow>>") {
        // rest = "addr,workspace_id,title,class" - we surface
        // "class,title" to match the activewindow shape.
        let parts: Vec<&str> = rest.splitn(4, ',').collect();
        if parts.len() == 4 {
            let payload = format!("{},{}", parts[3], parts[2]);
            return Some(HyprlandEvent::WindowOpened(payload));
        }
    }
    if let Some(rest) = line.strip_prefix("closewindow>>") {
        return Some(HyprlandEvent::WindowClosed(rest.to_string()));
    }
    None
}

// ---------------------------------------------------------------------
// Tests - pure parser checks, no socket needed.
// ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_workspace_change() {
        assert_eq!(
            parse_event("workspace>>3\n"),
            Some(HyprlandEvent::WorkspaceChanged(3))
        );
    }

    #[test]
    fn parse_activewindow() {
        let ev = parse_event("activewindow>>kitty,astra@host: ~").unwrap();
        match ev {
            HyprlandEvent::ActiveWindowChanged(s) => {
                assert_eq!(s, "kitty,astra@host: ~");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn parse_openwindow() {
        let ev = parse_event("openwindow>>0x1234,2,Firefox,firefox").unwrap();
        match ev {
            HyprlandEvent::WindowOpened(s) => {
                assert_eq!(s, "firefox,Firefox");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn parse_closewindow() {
        let ev = parse_event("closewindow>>0xdeadbeef\n").unwrap();
        match ev {
            HyprlandEvent::WindowClosed(s) => {
                assert_eq!(s, "0xdeadbeef");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn parse_unknown_event_returns_none() {
        assert!(parse_event("screencast>>1,1").is_none());
        assert!(parse_event("garbage").is_none());
        assert!(parse_event("").is_none());
    }
}
