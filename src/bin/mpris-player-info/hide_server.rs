use std::future::pending;

use anyhow::Context;
use tracing::{Level, info};
use tracing_subscriber::{EnvFilter,registry, layer::SubscriberExt, util::SubscriberInitExt};
use zbus::ConnectionBuilder;

use mpris_dbus::proxies::HideServer;

pub fn main() -> anyhow::Result<()> {
    let level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };
    registry().with(tracing_journald::layer().context("obtaining journald layer")?).with(EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy())
        .init();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build().context("building tokio runtime")?;
    rt.block_on(async {
        let server = HideServer::new(false);
        let _conn = ConnectionBuilder::session().context("choosing session dbus")?
            .name("com.github.robinmarchart.mprisutils").context("setting dbus name")?
            .serve_at("/com/github/robinmarchart/mprisutils", server).context("setting server")?
            .build()
            .await.context("setting up connection")?;
        info!("connection established");
        loop{
        pending::<()>().await
        }
    })
}
