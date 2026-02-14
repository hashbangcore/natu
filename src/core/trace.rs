use std::path::{Path, PathBuf};
use tokio::net::UnixDatagram;

const DEFAULT_TRACE_SOCKET_PATH: &str = "/tmp/netero.trace.sock";

fn resolve_trace_socket_path() -> PathBuf {
    if let Ok(value) = std::env::var("TRACE_SOCKET_PATH") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    if let Ok(value) = std::env::var("XDG_RUNTIME_DIR") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Path::new(trimmed).join("netero.trace.sock");
        }
    }

    PathBuf::from(DEFAULT_TRACE_SOCKET_PATH)
}

pub async fn run_trace_server() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = resolve_trace_socket_path();

    // Replace old socket if it exists.
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let socket = UnixDatagram::bind(&socket_path)?;
    let mut buf = vec![0u8; 64 * 1024];

    loop {
        let (len, _) = socket.recv_from(&mut buf).await?;
        let payload = String::from_utf8_lossy(&buf[..len]);
        println!("{}", payload);
    }
}

pub async fn send_trace(kind: &str, payload: &str) {
    let socket_path = resolve_trace_socket_path();
    let socket = match UnixDatagram::unbound() {
        Ok(sock) => sock,
        Err(_) => return,
    };

    let message = format!("{}\n{}", kind, payload);
    let _ = socket.send_to(message.as_bytes(), socket_path).await;
}
