#[cfg(feature = "info_polybar_yambar")]
use std::{pin::pin, sync::Arc, fs::{File, create_dir_all}, path::PathBuf, env, process};

use anyhow::Context;

#[cfg(feature = "info_polybar_yambar")]
use anyhow::anyhow;
use clap::Subcommand;
#[cfg(feature = "info_polybar_yambar")]
use mpris_dbus::player_info::PlayerInfo;
#[cfg(feature = "info_polybar_yambar")]
use zbus::{export::futures_util::StreamExt, Connection};

#[cfg(feature = "info_polybar_yambar")]
use time::macros::format_description;
use tracing::Level;
#[cfg(feature = "info_polybar_yambar")]
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::EnvFilter;
#[cfg(feature = "info_waybar")]
use tracing_subscriber::{layer::SubscriberExt, registry, util::SubscriberInitExt};

#[cfg(feature = "info_polybar_yambar")]
mod polybar;
#[cfg(feature = "info_waybar")]
mod waybar;
#[cfg(feature = "info_polybar_yambar")]
mod yambar;

#[derive(Subcommand, Clone)]
pub enum Bars {
    #[cfg(feature = "info_polybar_yambar")]
    Polybar {
        icon_font: u8,
        hide_cmd: String,
        name_len: u8,
    },
    #[cfg(feature = "info_polybar_yambar")]
    Yambar,
    #[cfg(feature = "info_waybar")]
    Waybar,
}

pub fn main(bar: Bars, hidden: bool) ->  anyhow::Result<()> {
    let level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };
    match bar {
        #[cfg(feature = "info_waybar")]
        Bars::Waybar => {
            registry()
                .with(tracing_journald::layer().context("getting journald layer")?)
                .with(
                    EnvFilter::builder()
                        .with_default_directive(level.into())
                        .from_env_lossy(),
                )
                .init();
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("building tokio runtime")?;
            rt.block_on(waybar::waybar(hidden))
        }
        #[cfg(feature = "info_polybar_yambar")]
        command @ Bars::Yambar
        | command @ Bars::Polybar {
            icon_font: _,
            hide_cmd: _,
            name_len: _,
        } => {
            let logdir = PathBuf::from(env::var_os("XDG_RUNTIME_DIR")
                                       .ok_or_else(|| anyhow!("XDG_RUNTIME_DIR not set"))?)
                .join("mpris-player-info");
            create_dir_all(&logdir).context("creating logdir")?;
            let logfile = File::create(logdir.join(format!("bar.{}.log",process::id())))
                .context("creating log file")?;
            tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::builder()
                        .with_default_directive(level.into())
                        .from_env_lossy(),
                )
                .event_format(tracing_subscriber::fmt::format().pretty().with_timer(
                    LocalTime::new(format_description!(
                        "[day].[month].[year] [hour]:[minute]:[second]:[subsecond digits:6]"
                    )),
                ))
                .with_writer(logfile)
                .init();
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("building tokio runtime")?;
            rt.block_on(async move {
                let mut stream = pin!(
                    mpris_dbus::hide::hidden_active_player_info(
                        &Connection::session().await?,
                        hidden
                    )
                    .await?
                );
                while let Some(info) = stream.next().await {
                    match &command {
                        Bars::Polybar {
                            icon_font,
                            hide_cmd,
                            name_len,
                        } => polybar::polybar(info, *icon_font, hide_cmd, *name_len),
                        Bars::Yambar => yambar::yambar(info),
                        #[cfg(feature = "info_waybar")]
                        Bars::Waybar => unreachable!(),
                    }
                }
                Ok(())
            })
        }
    }
}

#[cfg(feature = "info_polybar_yambar")]
type Info = Option<Option<Result<(Arc<Vec<String>>, PlayerInfo), Arc<zbus::Error>>>>;
