use std::error::Error;
#[cfg(feature = "polybar_yambar")]
use std::{sync::Arc, pin::pin};

use clap::{Parser, Subcommand};
#[cfg(feature = "polybar_yambar")]
use mpris_dbus::player_info::PlayerInfo;
#[cfg(feature = "polybar_yambar")]
use zbus::{export::futures_util::StreamExt, Connection};

#[cfg(feature = "polybar_yambar")]
use time::macros::format_description;
use tracing::Level;
#[cfg(feature = "waybar")]
use tracing_subscriber::{
    layer::SubscriberExt, registry, util::SubscriberInitExt,
};
use tracing_subscriber::EnvFilter;
#[cfg(feature = "polybar_yambar")]
use tracing_subscriber::fmt::time::LocalTime;

#[cfg(feature = "polybar_yambar")]
mod polybar;
#[cfg(feature = "waybar")]
mod waybar;
#[cfg(feature = "polybar_yambar")]
mod yambar;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short = 'd', long)]
    hidden: bool,
}

#[derive(Subcommand, Clone)]
enum Commands {
    #[cfg(feature = "polybar_yambar")]
    Polybar {
        icon_font: u8,
        hide_cmd: String,
        name_len: u8,
    },
    #[cfg(feature = "polybar_yambar")]
    Yambar,
    #[cfg(feature = "waybar")]
    Waybar,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };
    match cli.command {
        #[cfg(feature = "waybar")]
        Commands::Waybar => {
            registry()
                .with(tracing_journald::layer()?)
                .with(
                    EnvFilter::builder()
                        .with_default_directive(level.into())
                        .from_env_lossy(),
                )
                .init();
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            rt.block_on(waybar::waybar(cli.hidden))
        }
        #[cfg(feature = "polybar_yambar")]
        command @ Commands::Yambar
        | command @ Commands::Polybar {
            icon_font: _,
            hide_cmd: _,
            name_len: _,
        } => {
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
                .with_writer(std::io::stderr)
                .init();
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            rt.block_on(async move {
                let mut stream = pin!(
                    mpris_dbus::hide::hidden_active_player_info(
                        &Connection::session().await?,
                        cli.hidden
                    )
                    .await?
                );
                while let Some(info) = stream.next().await {
                    match &command {
                        Commands::Polybar {
                            icon_font,
                            hide_cmd,
                            name_len,
                        } => polybar::polybar(info, *icon_font, hide_cmd, *name_len),
                        Commands::Yambar => yambar::yambar(info),
    #[cfg(feature = "waybar")]
                        _ => unreachable!(),
                    }
                }
                Ok(())
            })
        }
    }
}

#[cfg(feature = "polybar_yambar")]
type Info = Option<Option<Result<(Arc<Vec<String>>, PlayerInfo), Arc<zbus::Error>>>>;
