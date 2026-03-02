use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use cncf_mcp_core::config::AppConfig;
use cncf_mcp_core::server::LogLevelReloadFn;

#[tokio::main]
async fn main() -> Result<()> {
    let default_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let (filter_layer, reload_handle) = tracing_subscriber::reload::Layer::new(default_filter);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_target(false)
                .with_writer(std::io::stderr),
        )
        .init();

    let log_level_reload: Option<LogLevelReloadFn> = Some(Box::new(move |level: &str| {
        let _ = reload_handle.modify(|filter| *filter = EnvFilter::new(level));
    }));

    let config = AppConfig::parse();

    info!(
        "CNCF MCP Server v{} starting (transport: {:?})",
        env!("CARGO_PKG_VERSION"),
        config.transport
    );

    cncf_mcp_core::server::run(config, log_level_reload).await
}
