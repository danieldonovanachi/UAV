mod bridge;
mod server;
mod simulation;
mod telemetry;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("exhibition=debug,info")
        .init();

    info!("Starting Exhibition Interface Server");

    let args: Vec<String> = std::env::args().collect();
    let port = args
        .get(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);

    let server = server::ExhibitionServer::new(port).await?;
    server.run().await?;

    Ok(())
}
