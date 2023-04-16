use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Serialize};
use zbus::{
    dbus_proxy,
    zvariant::{ObjectPath, OwnedValue, Type, Value, Str},
    Result,
};

#[cfg(feature = "hide")]
#[dbus_proxy(
    default_service = "com.github.robinmarchart.mprisutils",
    interface = "com.github.robinmarchart.mprisutils",
    default_path = "/com/github/robinmarchart/mprisutils"
)]
pub trait StateServer {
    fn toggle(&self) -> Result<bool>;
    #[dbus_proxy(property)]
    fn suppressed(&self) -> Result<bool>;
    #[dbus_proxy(property)]
    fn set_suppressed(&self, val: bool) -> Result<()>;
}

#[cfg(feature = "playerctld")]
#[dbus_proxy(
    interface = "com.github.altdesktop.playerctld",
    default_service = "org.mpris.MediaPlayer2",
    default_path = "org/mpris/MediaPlayer2"
)]
pub trait Playerctld {
    /// Shift method
    fn shift(&self) -> Result<String>;

    /// Unshift method
    fn unshift(&self) -> Result<String>;

    /// ActivePlayerChangeBegin signal
    #[dbus_proxy(signal)]
    fn active_player_change_begin(&self, name: &str) -> Result<()>;

    /// ActivePlayerChangeEnd signal
    #[dbus_proxy(signal)]
    fn active_player_change_end(&self, name: &str) -> Result<()>;

    /// PlayerNames property
    #[dbus_proxy(property)]
    fn player_names(&self) -> Result<Vec<String>>;
}

#[derive(Deserialize, Serialize, Type, PartialEq, Debug,Clone, Copy)]
#[zvariant(signature = "s")]
pub enum LoopStatus {
    None,
    Track,
    Playlist,
}

impl<'a> TryFrom<&'a Value<'a>> for LoopStatus {
    type Error=zbus::zvariant::Error;
    fn try_from(value: &'a Value<'a>) -> std::result::Result<Self, Self::Error> {
        Ok(match <&str as TryFrom<&'a Value<'a>>>::try_from(value)?{
            "None"=>LoopStatus::None,
            "Track"=>LoopStatus::Track,
            "Playlist"=>LoopStatus::Playlist,
            _=>return Err(zbus::zvariant::Error::Message("Unknown LoopStatus value".to_string()))
        })
    }
}
impl TryFrom<OwnedValue> for LoopStatus{
    type Error=zbus::zvariant::Error;
    fn try_from(value: OwnedValue) -> std::result::Result<Self, Self::Error> {
        LoopStatus::try_from(value.deref())
    }
}

impl<'a> From<LoopStatus> for Value<'a>{
    fn from(val: LoopStatus) -> Self {
        let str=match val {
            LoopStatus::None => "None",
            LoopStatus::Track => "Track",
            LoopStatus::Playlist => "Playlist",
        };
        Value::Str(Str::from_static(str))
    }
}

#[derive(Deserialize, Serialize, Type, PartialEq, Debug,Clone,Copy)]
#[zvariant(signature = "s")]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

impl<'a> TryFrom<&'a Value<'a>> for PlaybackStatus{
    type Error=zbus::zvariant::Error;
    fn try_from(value: &'a Value<'a>) -> std::result::Result<Self, Self::Error> {
        Ok(match <&str as TryFrom<&'a Value<'a>>>::try_from(value)?{
            "Playing"=>PlaybackStatus::Playing,
            "Paused"=>PlaybackStatus::Paused,
            "Stopped"=>PlaybackStatus::Stopped,
            _=>return Err(zbus::zvariant::Error::Message("Unknown PlaybackStatus value".to_string()))
        })
    }
}

impl TryFrom<OwnedValue> for PlaybackStatus {
    type Error=zbus::zvariant::Error;
    fn try_from(value: OwnedValue) -> std::result::Result<Self, Self::Error> {
        PlaybackStatus::try_from(value.deref())
    }
}

#[dbus_proxy(interface = "org.mpris.MediaPlayer2", assume_defaults = true)]
pub trait MediaPlayer2 {
    /// Quit method
    fn quit(&self) -> Result<()>;

    /// Raise method
    fn raise(&self) -> Result<()>;

    /// CanQuit property
    #[dbus_proxy(property)]
    fn can_quit(&self) -> Result<bool>;

    /// CanRaise property
    #[dbus_proxy(property)]
    fn can_raise(&self) -> Result<bool>;

    /// CanSetFullscreen property
    #[dbus_proxy(property)]
    fn can_set_fullscreen(&self) -> Result<bool>;

    /// DesktopEntry property
    #[dbus_proxy(property)]
    fn desktop_entry(&self) -> Result<String>;

    /// Fullscreen property
    #[dbus_proxy(property)]
    fn fullscreen(&self) -> Result<bool>;
    #[dbus_proxy(property)]
    fn set_fullscreen(&self, value: bool) -> Result<()>;

    /// HasTrackList property
    #[dbus_proxy(property)]
    fn has_track_list(&self) -> Result<bool>;

    /// Identity property
    #[dbus_proxy(property)]
    fn identity(&self) -> Result<String>;

    /// SupportedMimeTypes property
    #[dbus_proxy(property)]
    fn supported_mime_types(&self) -> Result<Vec<String>>;

    /// SupportedUriSchemes property
    #[dbus_proxy(property)]
    fn supported_uri_schemes(&self) -> Result<Vec<String>>;
}

#[dbus_proxy(interface = "org.mpris.MediaPlayer2.Player", assume_defaults = true)]
pub trait Player {
    /// Next method
    fn next(&self) -> Result<()>;

    /// OpenUri method
    fn open_uri(&self, uri: &str) -> Result<()>;

    /// Pause method
    fn pause(&self) -> Result<()>;

    /// Play method
    fn play(&self) -> Result<()>;

    /// PlayPause method
    fn play_pause(&self) -> Result<()>;

    /// Previous method
    fn previous(&self) -> Result<()>;

    /// Seek method
    fn seek(&self, offset: i64) -> Result<()>;

    /// SetPosition method
    fn set_position(&self, track_id: &ObjectPath<'_>, offset: i64) -> Result<()>;

    /// Stop method
    fn stop(&self) -> Result<()>;

    /// Seeked signal
    #[dbus_proxy(signal)]
    fn seeked(&self, position: i64) -> Result<()>;

    /// CanControl property
    #[dbus_proxy(property)]
    fn can_control(&self) -> Result<bool>;

    /// CanGoNext property
    #[dbus_proxy(property)]
    fn can_go_next(&self) -> Result<bool>;

    /// CanGoPrevious property
    #[dbus_proxy(property)]
    fn can_go_previous(&self) -> Result<bool>;

    /// CanPause property
    #[dbus_proxy(property)]
    fn can_pause(&self) -> Result<bool>;

    /// CanPlay property
    #[dbus_proxy(property)]
    fn can_play(&self) -> Result<bool>;

    /// CanSeek property
    #[dbus_proxy(property)]
    fn can_seek(&self) -> Result<bool>;

    /// LoopStatus property
    #[dbus_proxy(property)]
    fn loop_status(&self) -> Result<LoopStatus>;
    #[dbus_proxy(property)]
    fn set_loop_status(&self, value: LoopStatus) -> Result<()>;

    /// MaximumRate property
    #[dbus_proxy(property)]
    fn maximum_rate(&self) -> Result<f64>;

    /// Metadata property
    #[dbus_proxy(property)]
    fn metadata(&self) -> Result<HashMap<String, OwnedValue>>;

    /// MinimumRate property
    #[dbus_proxy(property)]
    fn minimum_rate(&self) -> Result<f64>;

    /// PlaybackStatus property
    #[dbus_proxy(property)]
    fn playback_status(&self) -> Result<PlaybackStatus>;

    /// Position property
    #[dbus_proxy(property)]
    fn position(&self) -> Result<i64>;

    /// Rate property
    #[dbus_proxy(property)]
    fn rate(&self) -> Result<f64>;
    #[dbus_proxy(property)]
    fn set_rate(&self, value: f64) -> Result<()>;

    /// Shuffle property
    #[dbus_proxy(property)]
    fn shuffle(&self) -> Result<bool>;
    #[dbus_proxy(property)]
    fn set_shuffle(&self, value: bool) -> Result<()>;

    /// Volume property
    #[dbus_proxy(property)]
    fn volume(&self) -> Result<f64>;
    #[dbus_proxy(property)]
    fn set_volume(&self, value: f64) -> Result<()>;
}
