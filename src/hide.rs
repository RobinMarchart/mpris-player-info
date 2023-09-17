#[cfg(feature = "hide_active_player_info")]
use std::sync::Arc;

use tracing::{debug_span, event, info, Instrument, Level};
use zbus::{
    export::futures_util::{Stream, StreamExt},
    Connection,
};

use crate::{
    proxies::HideStateProxy,
    util::{poll_both, ResultExt, StreamExt2},
};

#[cfg(feature = "hide_active_player_info")]
use crate::{active_player_info::active_player_info, player_info::PlayerInfo};

use std::fmt::Debug;

pub async fn hidden(conn: &Connection) -> zbus::Result<impl Stream<Item = zbus::Result<bool>>> {
    let span = debug_span!("hidden");
    async {
        let proxy = HideStateProxy::new(conn).await?;
        let stream = proxy.receive_hidden_changed().await;
        info!("connected to hide state server");
        Ok(stream
            .then(|event| async move { event.get().await })
            .with_initial_value(proxy.hidden().await).filter_no_change())
    }
    .instrument(span.clone())
    .await
    .trace_err_span(&span)
    .map(|s| {
        s.inspect(|r| event!(Level::DEBUG,hidden = ?r))
            .instrument_stream(span)
    })
}

pub async fn hide<S: Stream>(
    conn: &Connection,
    s: S,
    default: bool,
) -> zbus::Result<impl Stream<Item = Option<S::Item>>>
where
    S::Item: Clone + Debug,
{
    let span = debug_span!("hidden");
    async move {
        Ok(poll_both(
            hidden(conn)
                .await?
                .map(move |v| v.unwrap_or(default)),
            s,
        )
        .map(move |(hidden, val)| {
            let hidden = hidden.unwrap_or(default);
            if hidden {
                None
            } else {
                val
            }
        }))
    }
    .instrument(span.clone())
    .await
    .trace_err_span(&span)
    .map(|s| {
        s.inspect(|r| event!(Level::DEBUG,hide = ?r))
            .instrument_stream(span)
    })
}

#[cfg(feature = "hide_active_player_info")]
pub async fn hidden_active_player_info(
    conn: &Connection,
    default: bool,
) -> zbus::Result<
    impl Stream<Item = Option<Option<Result<(Arc<Vec<String>>, PlayerInfo), Arc<zbus::Error>>>>>,
> {
    let span = debug_span!("hide_active_player_info");
    async move {
        let info = active_player_info(conn.clone())
            .await?
            .map(|v| v.map(|v| v.map_err(Arc::new)));
        hide(conn, info, default).await
    }
    .instrument(span.clone())
    .await
    .trace_err_span(&span)
    .map(|s| {
        s.filter_no_change().inspect(|r| event!(Level::DEBUG,hide_active_player_info = ?r))
            .instrument_stream(span)
    })
}
