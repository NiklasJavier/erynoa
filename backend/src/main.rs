//! Erynoa Backend - Binary Entrypoint

#[cfg(all(not(target_env = "msvc"), feature = "jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use erynoa_api::{
    config::{version::VERSION, Settings},
    server::Server,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("erynoa-backend".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = Settings::load().expect("Failed to load configuration");

    tracing::info!(
        version = VERSION,
        env = %settings.application.environment.as_str(),
        port = settings.application.port,
        "🚀 Starting Erynoa API"
    );

    let server = Server::build(settings).await?;
    server.run().await?;

    Ok(())
}
