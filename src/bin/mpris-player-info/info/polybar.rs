use std::borrow::Cow;

use super::Info;



pub fn polybar(info: Info, icon_font: u8, hide_cmd: &str, name_len: u8) {
    match info {
        Some(info) => match info {
            Some(info) => match info {
                Ok((names, info)) => {
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
                    if names.len()!=1{
                    println!(
                        "%{{T{icon_font}}}%{{A1:{hide_cmd}:}}🐧%{{A}}%{{T-}} \
                         %{{T{icon_font}}}%{{A1:playerctld shift:}}⏶%{{A}}%{{T-}} \
                         %{{T{icon_font}}}%{{A1:playerctld unshift:}}⏷%{{A}}%{{T-}} \
                         {name} %{{T{icon_font}}}%{{A1:playerctl play-pause:}}{play}%{{A}}%{{T-}} \
                         %{{T{icon_font}}}%{{A1:playerctl previous:}}⏮%{{A}}%{{T-}} \
                         %{{T{icon_font}}}%{{A1:playerctl next:}}⏭%{{A}}%{{T-}}"
                    )
                    }else{
                     println!(
                        "%{{T{icon_font}}}%{{A1:{hide_cmd}:}}🐧%{{A}}%{{T-}} \
                         {name} %{{T{icon_font}}}%{{A1:playerctl play-pause:}}{play}%{{A}}%{{T-}} \
                         %{{T{icon_font}}}%{{A1:playerctl previous:}}⏮%{{A}}%{{T-}} \
                         %{{T{icon_font}}}%{{A1:playerctl next:}}⏭%{{A}}%{{T-}}"
                    )
                    }

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
