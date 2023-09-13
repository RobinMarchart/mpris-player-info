use std::{borrow::Cow, error::Error, pin::pin, sync::Arc};

use clap::{Parser, Subcommand};
use mpris_dbus::player_info::PlayerInfo;
use zbus::{export::futures_util::StreamExt, Connection};

use time::macros::format_description;
use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::time::LocalTime, EnvFilter};
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
    Polybar {
        icon_font: u8,
        hide_cmd: String,
        name_len: u8,
    },
    Yambar,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let level = if cfg!(debug_assertions){
        Level::DEBUG
    }else {
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
            ).with_writer(std::io::stderr)
            .init();
    LogTracer::init()?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let mut stream = pin!(
            mpris_dbus::hide::hidden_active_player_info(&Connection::session().await?, cli.hidden)
                .await?
        );
        while let Some(info) = stream.next().await {
            match &cli.command {
                Commands::Polybar {
                    icon_font,
                    hide_cmd,
                    name_len,
                } => polybar(info, *icon_font, hide_cmd, *name_len),
                Commands::Yambar => yambar(info),
            }
        }
        Ok(())
    })
}

type Info = Option<Option<Result<(&'static str, PlayerInfo), Arc<zbus::Error>>>>;

fn polybar(info: Info, icon_font: u8, hide_cmd: &str, name_len: u8) {
    match info {
        Some(info) => match info {
            Some(info) => match info {
                Ok((_, info)) => {
                    let mut artist: Option<&str> = info
                        .metadata
                        .get("xesam:artist")
                        .and_then(|i| i.downcast_ref());
                    let title: Option<&str> = info
                        .metadata
                        .get("xesam:title")
                        .and_then(|i| i.downcast_ref());
                    if artist.is_some()
                        && title.is_some()
                        && title.unwrap().starts_with(artist.unwrap())
                    {
                        artist = None;
                    }
                    let mut name = match (artist, title) {
                        (Some(name), Some(title)) => Cow::from(format!("{name} - {title}")),
                        (Some(name), None) => Cow::from(name),
                        (None, Some(title)) => Cow::from(title),
                        (None, None) => Cow::from(""),
                    };

                    if let Some((n, _)) = name.char_indices().nth(name_len as usize - 1) {
                        name = Cow::from(format!("{}…", &name[0..n]));
                    }

                    let play = match info.playback_status {
                        mpris_dbus::proxies::PlaybackStatus::Playing => "⏸",
                        mpris_dbus::proxies::PlaybackStatus::Paused => "▶",
                        mpris_dbus::proxies::PlaybackStatus::Stopped => "⬛",
                    };
                    println!(
                        "%{{T{icon_font}}}%{{A1:{hide_cmd}:}}🐧%{{A}}%{{T-}} \
                         %{{T{icon_font}}}%{{A1:playerctld shift:}}⏶%{{A}}%{{T-}} \
                         %{{T{icon_font}}}%{{A1:playerctld unshift:}}⏷%{{A}}%{{T-}} \
                         {name} %{{T{icon_font}}}%{{A1:playerctl play-pause:}}{play}%{{A}}%{{T-}} \
                         %{{T{icon_font}}}%{{A1:playerctl previous:}}⏮%{{A}}%{{T-}} \
                         %{{T{icon_font}}}%{{A1:playerctl next:}}⏭%{{A}}%{{T-}}"
                    )
                }
                Err(e) => {
                    println!("{e}")
                }
            },
            None => println!("%{{T{icon_font}}}%{{A1:{hide_cmd}:}}🐧%{{A}}%{{T-}} No Player"),
        },
        None => println!("%{{T{icon_font}}}%{{A1:{hide_cmd}:}}🐧%{{A}}%{{T-}}"),
    }
}

struct YambarInfo<'a> {
    show: bool,
    switch: bool,
    text: Cow<'a, str>,
    next: bool,
    prev: bool,
    show_play: bool,
    play_pause: &'static str,
}

fn yambar(info: Info) {
    let info = match info.as_ref() {
        Some(info) => match info.as_ref() {
            Some(info) => match info.as_ref() {
                Ok((_, info)) => {
                    let mut artist: Option<&str> = info
                        .metadata
                        .get("xesam:artist")
                        .and_then(|i| i.downcast_ref());
                    let title: Option<&str> = info
                        .metadata
                        .get("xesam:title")
                        .and_then(|i| i.downcast_ref());
                    if artist.is_some()
                        && title.is_some()
                        && title.unwrap().starts_with(artist.unwrap())
                    {
                        artist = None;
                    }
                    let name = match (artist, title) {
                        (Some(name), Some(title)) => Cow::from(format!("{name} - {title}")),
                        (Some(name), None) => Cow::from(name),
                        (None, Some(title)) => Cow::from(title),
                        (None, None) => Cow::from(""),
                    };
                    let play = match info.playback_status {
                        mpris_dbus::proxies::PlaybackStatus::Playing => "⏸",
                        mpris_dbus::proxies::PlaybackStatus::Paused => "▶",
                        mpris_dbus::proxies::PlaybackStatus::Stopped => "⬛",
                    };

                    YambarInfo {
                        show: true,
                        switch: true,
                        text: name,
                        prev: info.can_go_previous,
                        next: info.can_go_next,
                        show_play: true,
                        play_pause: play,
                    }
                }
                Err(e) => YambarInfo {
                    show: true,
                    switch: false,
                    text: Cow::Owned(format!("{e}")),
                    prev: false,
                    next: false,
                    show_play: false,
                    play_pause: "",
                },
            },
            None => YambarInfo {
                show: true,
                switch: false,
                text: Cow::Borrowed("No Player"),
                prev: false,
                next: false,
                show_play: false,
                play_pause: "",
            },
        },
        None => YambarInfo {
            show: false,
            switch: false,
            text: Cow::Borrowed(""),
            next: false,
            prev: false,
            show_play: false,
            play_pause: "",
        },
    };
    println!(
        "\
mpris_show|bool|{}
mpris_switch|bool|{}
mpris_text|string|{}
mpris_next|bool|{}
mpris_prev|bool|{}
mpris_show_play|bool|{}
mpris_play_pause|string|{}
",
        info.show, info.switch, info.text, info.next, info.prev, info.show_play, info.play_pause
    );
}
