use crate::{
    proxies::PlayerctldProxy,
    util::{ResultExt, StreamExt2},
};
use tracing::{debug_span, event, info, Instrument, Level};
use zbus::{
    export::futures_util::{Stream, StreamExt},
    Connection,
};

pub async fn active_players(
    conn: &Connection,
) -> zbus::Result<impl Stream<Item = zbus::Result<Vec<String>>>> {
    let span = debug_span!("active_players");
    async {
        let proxy: PlayerctldProxy<'static> = PlayerctldProxy::new(conn).await?;
        let changes = proxy.receive_player_names_changed().await;
        info!("connected to playerctld");
        Ok(changes
            .then(|event| async move { event.get().await })
            .with_initial_value(proxy.player_names().await).filter_no_change())
    }
    .instrument(span.clone())
    .await
    .trace_err_span(&span)
    .map(|s| {
        s.inspect(|r| event!(Level::DEBUG,active_players = ?r))
            .instrument_stream(span)
    })
}
