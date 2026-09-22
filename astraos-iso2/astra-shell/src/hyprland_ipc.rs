use log::{error, info};
use std::env;
use std::path::PathBuf;
use std::thread;
use tokio::net::UnixStream;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum HyprlandEvent {
    WorkspaceChanged(u32),
    ActiveWindowChanged(String),
    WindowOpened(String),
    WindowClosed(String),
}

pub struct HyprlandClient {
    socket_path: Option<PathBuf>,
}

impl HyprlandClient {
    pub fn new() -> Self {
        Self {
            socket_path: Self::find_event_socket(),
        }
    }

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

async fn listen_events(mut stream: UnixStream) -> std::io::Result<()> {
    use tokio::io::AsyncBufReadExt;

    let mut reader = tokio::io::BufReader::new(&mut stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, 
            Ok(_) => {
                if let Some(ev) = parse_event(&line) {
                    info!("Hyprland event: {:?}", ev);
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
