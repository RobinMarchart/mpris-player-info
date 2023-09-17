use std::{future::ready, sync::Arc};

use crate::{
    active_player::active_players,
    player_info::{player_info, PlayerInfo},
    util::{string_to_static, ResultExt, StreamExt2},
};
use tracing::{debug_span, event, info, Instrument, Level};
use zbus::{
    export::futures_util::{stream::once, Stream, StreamExt},
    Connection,
};

pub async fn active_player_info(
    conn: Connection,
) -> zbus::Result<impl Stream<Item = Option<zbus::Result<(Arc<Vec<String>>, PlayerInfo)>>>> {
    let span = debug_span!("active_player_info");
    async move {
        let stream = active_players(&conn)
            .await?
            .then(move |names| {
                let conn = conn.clone();
                async move {
                    match names {
                        Ok(names) => {
                            if names.is_empty() {
                                info!("no active player");
                                once(ready(None)).right_stream().right_stream()
                            } else {
                                let name: &'static str = string_to_static(names[0].clone());
                                info!("new active player: {name}");
                                let info = player_info(name, &conn);
                                let names = Arc::new(names);
                                match info.await {
                                    Ok(info) => {
                                        info.map(move |i| Some(i.map(|i| (names.clone(), i)))).right_stream()
                                    }
                                    Err(e) => once(ready(Some(Err(e)))).left_stream(),
                                }
                                .left_stream()
                                .right_stream()
                            }
                        }
                        Err(e) => once(ready(Some(Err(e)))).left_stream(),
                    }
                }
            })
            .flatten_newest().filter_no_change();
        Ok(stream)
    }
    .instrument(span.clone())
    .await
    .trace_err_span(&span)
    .map(|s| {
        s.inspect(|r| event!(Level::DEBUG,active_player_info = ?r))
            .instrument_stream(span)
    })
}
