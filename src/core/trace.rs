use std::path::Path;
use tokio::net::UnixDatagram;

pub const TRACE_SOCKET_PATH: &str = "/tmp/netero.trace.sock";

pub async fn run_trace_server() -> Result<(), Box<dyn std::error::Error>> {
    if Path::new(TRACE_SOCKET_PATH).exists() {
        std::fs::remove_file(TRACE_SOCKET_PATH)?;
    }

    let socket = UnixDatagram::bind(TRACE_SOCKET_PATH)?;
    let mut buf = vec![0u8; 64 * 1024];

    loop {
        let (len, _) = socket.recv_from(&mut buf).await?;
        let payload = String::from_utf8_lossy(&buf[..len]);
        println!("{}", payload);
    }
}

pub async fn send_trace(kind: &str, payload: &str) {
    let socket = match UnixDatagram::unbound() {
        Ok(sock) => sock,
        Err(_) => return,
    };

    let message = format!("{}\n{}", kind, payload);
    let _ = socket
        .send_to(message.as_bytes(), TRACE_SOCKET_PATH)
        .await;
}
