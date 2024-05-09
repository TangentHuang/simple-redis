use anyhow::Result;
use simple_redis::stream_handle;
use simple_redis::Backend;
use tracing::{info, warn};
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "127.0.0.1:6379";
    info!("[Simple-redis-server]listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let backend = Backend::new();
    loop {
        let (socket, raddr) = listener.accept().await?;
        info!("[Simple-redis-server]accepted connection from {}", raddr);
        let cloned_backend = backend.clone();
        tokio::spawn(async move {
            if let Err(e) = stream_handle(socket, cloned_backend).await {
                warn!(
                    "[Simple-redis-server]error processing connection from {}: {:?}",
                    raddr, e
                );
            }
        });
    }
}
