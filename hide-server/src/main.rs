use std::{error::Error, future::pending};

use tracing::{Level, info};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::time::LocalTime, EnvFilter};
use zbus::ConnectionBuilder;

use mpris_dbus::proxies::HideServer;
use time::macros::format_description;

fn main() -> Result<(), Box<dyn Error>> {
    let level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy(),
        )
        .event_format(
            tracing_subscriber::fmt::format()
                .pretty()
                .with_timer(LocalTime::new(format_description!(
                    "[day].[month].[year] [hour]:[minute]:[second]:[subsecond digits:6]"
                ))),
        ).init();
    LogTracer::init()?;
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
