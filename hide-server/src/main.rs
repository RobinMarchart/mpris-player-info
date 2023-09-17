use std::{error::Error, future::pending};

use tracing::{Level, info};
use tracing_subscriber::{EnvFilter,registry, layer::SubscriberExt, util::SubscriberInitExt};
use zbus::ConnectionBuilder;

use mpris_dbus::proxies::HideServer;

fn main() -> Result<(), Box<dyn Error>> {
    let level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };
    registry().with(tracing_journald::layer()?).with(EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy())
        .init();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let server = HideServer::new(false);
        let _conn = ConnectionBuilder::session()?
            .name("com.github.robinmarchart.mprisutils")?
            .serve_at("/com/github/robinmarchart/mprisutils", server)?
            .build()
            .await?;
        info!("connection established");
        loop{
        pending::<()>().await
        }
    })
}
