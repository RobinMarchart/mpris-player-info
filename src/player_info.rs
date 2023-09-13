use std::{collections::HashMap, sync::Arc};

use tracing::{debug_span, event, info, Instrument, Level};
use zbus::{
    export::futures_util::{
        stream::{empty, select},
        Stream, StreamExt,
    },
    zvariant::OwnedValue,
    Connection,
};

use crate::{
    proxies::{LoopStatus, PlaybackStatus, PlayerProxy},
    util::{ResultExt, StreamExt2},
};

#[derive(Debug, Clone,PartialEq)]
pub struct PlayerInfo {
    pub playback_status: PlaybackStatus,
    pub loop_status: Option<LoopStatus>,
    pub shuffle: Option<bool>,
    pub metadata: Arc<HashMap<String, OwnedValue>>,
    pub can_go_next: bool,
    pub can_go_previous: bool,
    pub can_play: bool,
    pub can_pause: bool,
}

#[derive(Debug)]
enum PlayerInfoUpdate {
    PlaybackStatus(PlaybackStatus),
    LoopStatus(LoopStatus),
    Shuffle(bool),
    Metadata(HashMap<String, OwnedValue>),
    CanGoNext(bool),
    CanGoPrevious(bool),
    CanPlay(bool),
    CanPause(bool),
    Nothing,
}

pub async fn player_info<'a>(
    name: &'a str,
    conn: &Connection,
) -> zbus::Result<impl Stream<Item = zbus::Result<PlayerInfo>> + 'a> {
    let span = debug_span!("player_info", name);
    async {
        let proxy = PlayerProxy::builder(conn)
            .destination(name)?
            .build()
            .await?;
        let (
            playback_status_stream,
            loop_status_stream,
            shuffle_stream,
            metadata_stream,
            can_go_next_stream,
            can_go_previous_stream,
            can_play_stream,
            can_pause_stream,
        ) = tokio::join!(
            proxy.receive_playback_status_changed(),
            proxy.receive_loop_status_changed(),
            proxy.receive_shuffle_changed(),
            proxy.receive_metadata_changed(),
            proxy.receive_can_go_next_changed(),
            proxy.receive_can_go_previous_changed(),
            proxy.receive_can_play_changed(),
            proxy.receive_can_pause_changed()
        );

        let playback_status_stream = playback_status_stream.then(|event| async move {
            zbus::Result::Ok(PlayerInfoUpdate::PlaybackStatus(event.get().await?))
        }).inspect(|v|event!(Level::DEBUG,playback_status = ?v)).instrument_stream(debug_span!("playback_status"));
        let loop_status_stream = loop_status_stream.then(|event| async move {
            zbus::Result::Ok(PlayerInfoUpdate::LoopStatus(event.get().await?))
        }).inspect(|v|event!(Level::DEBUG,loop_status = ?v)).instrument_stream(debug_span!("loop_status"));
        let shuffle_stream = shuffle_stream.then(|event| async move {
            zbus::Result::Ok(PlayerInfoUpdate::Shuffle(event.get().await?))
        }).inspect(|v|event!(Level::DEBUG,shuffle = ?v)).instrument_stream(debug_span!("shuffle"));
        let metadata_stream = metadata_stream.then(|event| async move {
            zbus::Result::Ok(PlayerInfoUpdate::Metadata(event.get().await?))
        }).inspect(|v|event!(Level::DEBUG,metadata = ?v)).instrument_stream(debug_span!("metadata"));
        let can_go_next_stream = can_go_next_stream.then(|event| async move {
            zbus::Result::Ok(PlayerInfoUpdate::CanGoNext(event.get().await?))
        }).inspect(|v|event!(Level::DEBUG,can_go_next = ?v)).instrument_stream(debug_span!("can_go_next"));
        let can_go_previous_stream = can_go_previous_stream.then(|event| async move {
            zbus::Result::Ok(PlayerInfoUpdate::CanGoPrevious(event.get().await?))
        }).inspect(|v|event!(Level::DEBUG,can_go_previous = ?v)).instrument_stream(debug_span!("can_go_previous"));
        let can_play_stream = can_play_stream.then(|event| async move {
            zbus::Result::Ok(PlayerInfoUpdate::CanPlay(event.get().await?))
        }).inspect(|v|event!(Level::DEBUG,can_play = ?v)).instrument_stream(debug_span!("can_play"));
        let can_pause_stream = can_pause_stream.then(|event| async move {
            zbus::Result::Ok(PlayerInfoUpdate::CanPause(event.get().await?))
        }).inspect(|v|event!(Level::DEBUG,can_pause = ?v)).instrument_stream(debug_span!("can_pause"));

        info!("connected to {name}");

        let (
            playback_status,
            loop_status,
            shuffle,
            metadata,
            can_go_next,
            can_go_previous,
            can_play,
            can_pause,
        ) = tokio::join!(
            proxy.playback_status(),
            proxy.loop_status(),
            proxy.shuffle(),
            proxy.metadata(),
            proxy.can_go_next(),
            proxy.can_go_previous(),
            proxy.can_play(),
            proxy.can_pause()
        );
        let info = PlayerInfo {
            playback_status: playback_status?,
            loop_status: loop_status.as_ref().ok().copied(),
            shuffle: shuffle.as_ref().ok().copied(),
            metadata: Arc::new(metadata?),
            can_go_next: can_go_next?,
            can_go_previous: can_go_previous?,
            can_play: can_play?,
            can_pause: can_pause?,
        };

        let loop_status_stream = if loop_status.is_ok() {
            loop_status_stream.right_stream()
        } else {
            empty().left_stream()
        };

        let shuffle_stream = if shuffle.is_ok() {
            shuffle_stream.right_stream()
        } else {
            empty().left_stream()
        };

        let update_stream = select(
            select(
                select(playback_status_stream, loop_status_stream),
                select(shuffle_stream, metadata_stream),
            ),
            select(
                select(can_go_next_stream, can_go_previous_stream),
                select(can_play_stream, can_pause_stream),
            ),
        )
        .with_initial_value(Ok(PlayerInfoUpdate::Nothing));

        let stream = update_stream.fold_map(info, |new, mut fold| {
            match new {
                Err(e) => return (Err(e), fold),
                Ok(PlayerInfoUpdate::PlaybackStatus(status)) => {
                    fold.playback_status = status;
                }
                Ok(PlayerInfoUpdate::LoopStatus(status)) => {
                    fold.loop_status = Some(status);
                }
                Ok(PlayerInfoUpdate::Shuffle(shuffle)) => {
                    fold.shuffle = Some(shuffle);
                }
                Ok(PlayerInfoUpdate::Metadata(metadata)) => {
                    fold.metadata = Arc::new(metadata);
                }
                Ok(PlayerInfoUpdate::CanGoNext(next)) => {
                    fold.can_go_next = next;
                }
                Ok(PlayerInfoUpdate::CanGoPrevious(previous)) => {
                    fold.can_go_previous = previous;
                }
                Ok(PlayerInfoUpdate::CanPlay(play)) => {
                    fold.can_play = play;
                }
                Ok(PlayerInfoUpdate::CanPause(pause)) => {
                    fold.can_pause = pause;
                }
                Ok(PlayerInfoUpdate::Nothing) => {}
            }
            let fold2 = fold.clone();
            (Ok(fold), fold2)
        }).filter_no_change();
        Ok(stream)
    }
    .instrument(span.clone())
    .await
    .trace_err_span(&span)
    .map(|s| {
        s.inspect(|r| event!(Level::DEBUG,player_info = ?r))
            .instrument_stream(span)
    })
}
