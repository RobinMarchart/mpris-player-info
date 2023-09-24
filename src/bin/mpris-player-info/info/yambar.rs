use std::borrow::Cow;

use super::Info;

struct YambarInfo<'a> {
    show: bool,
    switch: bool,
    text: Cow<'a, str>,
    next: bool,
    prev: bool,
    show_play: bool,
    play_pause: &'static str,
}

pub fn yambar(info: Info) {
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
