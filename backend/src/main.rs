//! Erynoa Backend - Binary Entrypoint

#[cfg(all(not(target_env = "msvc"), feature = "jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use erynoa_api::{
    config::{version::VERSION, Settings},
    server::Server,
    telemetry::{get_subscriber, init_subscriber},
};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("erynoa-backend".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = Settings::load().expect("Failed to load configuration");

    // Parse CLI arguments for static file serving
    // Usage: erynoa-api [--static-dir <path>]
    let args: Vec<String> = env::args().collect();
    let static_dir = parse_static_dir(&args);

    tracing::info!(
        version = VERSION,
        env = %settings.application.environment.as_str(),
        port = settings.application.port,
        static_dir = ?static_dir,
        "🚀 Starting Erynoa API"
    );

    let server = Server::build_with_static(settings, static_dir.as_deref()).await?;
    server.run().await?;

    Ok(())
}

/// Parse --static-dir argument from CLI
fn parse_static_dir(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--static-dir" {
            return iter.next().cloned();
        }
        if let Some(path) = arg.strip_prefix("--static-dir=") {
            return Some(path.to_string());
        }
    }

    // Fallback: Check environment variable
    env::var("ERYNOA_STATIC_DIR").ok()
}
