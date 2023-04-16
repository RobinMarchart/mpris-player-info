use std::{
    collections::HashMap, future::Future, marker::PhantomPinned, mem::transmute, pin::Pin,
    sync::Arc, task::Poll,
};

use zbus::{
    export::futures_util::Stream, zvariant::OwnedValue, Connection, PropertyChanged, PropertyStream,
};

use crate::proxies::{LoopStatus, PlaybackStatus, PlayerProxy};

struct PropertyS<T, F, FUT>
where
    T: 'static,
    F: Fn(&'static PropertyChanged<'static, T>) -> FUT,
    FUT: Future<Output = zbus::Result<T>>,
{
    stream: Option<PropertyStream<'static, T>>,
    stream_changed: Option<PropertyChanged<'static, T>>,
    stream_future: Option<FUT>,
    stream_fun: F,
    _pin: PhantomPinned,
}

impl<
        T: 'static,
        F: Fn(&'static PropertyChanged<'static, T>) -> FUT,
        FUT: Future<Output = zbus::Result<T>>,
    > Drop for PropertyS<T, F, FUT>
{
    fn drop(&mut self) {
        self.stream_future = None;
        self.stream_changed = None;
        self.stream = None;
    }
}

fn property_stream<E: Into<zbus::Error>, T: 'static + TryFrom<OwnedValue, Error = E> + Unpin>(
    from: Option<PropertyStream<'static, T>>,
) -> impl Stream<Item = zbus::Result<T>> {
    PropertyS {
        stream: from,
        stream_changed: None,
        stream_future: None,
        stream_fun: |e| e.get(),
        _pin: PhantomPinned,
    }
}

unsafe fn extend_lifetime<T>(v: &T) -> &'static T {
    transmute(v)
}

impl<
        E: Into<zbus::Error>,
        T: 'static + TryFrom<OwnedValue, Error = E> + Unpin,
        F: Fn(&'static PropertyChanged<'static, T>) -> FUT,
        FUT: Future<Output = zbus::Result<T>>,
    > Stream for PropertyS<T, F, FUT>
{
    type Item = zbus::Result<T>;
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        unsafe {
            let inner = self.get_unchecked_mut();
            match inner.stream_future.as_mut() {
                Some(f) => match Pin::new_unchecked(f).poll(cx) {
                    Poll::Ready(v) => {
                        inner.stream_future = None;
                        inner.stream_changed = None;
                        Poll::Ready(Some(v))
                    }
                    Poll::Pending => Poll::Pending,
                },
                None => match inner.stream.as_mut() {
                    Some(v) => match Pin::new_unchecked(v).poll_next(cx) {
                        Poll::Ready(Some(v)) => {
                            inner.stream_changed = Some(v);
                            inner.stream_future = Some((inner.stream_fun)(extend_lifetime(
                                inner.stream_changed.as_ref().unwrap_unchecked(),
                            )));
                            match Pin::new_unchecked(
                                inner.stream_future.as_mut().unwrap_unchecked(),
                            )
                            .poll(cx)
                            {
                                Poll::Ready(v) => {
                                    inner.stream_future = None;
                                    inner.stream_changed = None;
                                    Poll::Ready(Some(v))
                                }
                                Poll::Pending => Poll::Pending,
                            }
                        }
                        Poll::Ready(None) => {
                            inner.stream = None;
                            Poll::Ready(None)
                        }
                        Poll::Pending => Poll::Pending,
                    },
                    None => Poll::Ready(None),
                },
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    playback_status: PlaybackStatus,
    loop_status: Option<LoopStatus>,
    shuffle: Option<bool>,
    metadata: Arc<HashMap<String, OwnedValue>>,
    can_go_next: bool,
    can_go_previous: bool,
    can_play: bool,
    can_pause: bool,
}

struct PlayerInfoInner<
    PS: Stream<Item = zbus::Result<PlaybackStatus>>,
    LS: Stream<Item = zbus::Result<LoopStatus>>,
    SH: Stream<Item = zbus::Result<bool>>,
    ME: Stream<Item = zbus::Result<HashMap<String, OwnedValue>>>,
    CGN: Stream<Item = zbus::Result<bool>>,
    CGP: Stream<Item = zbus::Result<bool>>,
    CPL: Stream<Item = zbus::Result<bool>>,
    CPA: Stream<Item = zbus::Result<bool>>,
> {
    conn: Connection,
    info: PlayerInfo,
    err: Vec<zbus::Error>,
    playback_status: Option<PS>,
    loop_status: Option<LS>,
    shuffle: Option<SH>,
    metadata: Option<ME>,
    can_go_next: Option<CGN>,
    can_go_previous: Option<CGP>,
    can_play: Option<CPL>,
    can_pause: Option<CPA>,
}

#[allow(clippy::type_complexity)]
struct PlayerInfoStream<
    PS: Stream<Item = zbus::Result<PlaybackStatus>>,
    LS: Stream<Item = zbus::Result<LoopStatus>>,
    SH: Stream<Item = zbus::Result<bool>>,
    ME: Stream<Item = zbus::Result<HashMap<String, OwnedValue>>>,
    CGN: Stream<Item = zbus::Result<bool>>,
    CGP: Stream<Item = zbus::Result<bool>>,
    CPL: Stream<Item = zbus::Result<bool>>,
    CPA: Stream<Item = zbus::Result<bool>>,
> {
    inner: Pin<Box<PlayerInfoInner<PS, LS, SH, ME, CGN, CGP, CPL, CPA>>>,
}

impl<
        PS: Stream<Item = zbus::Result<PlaybackStatus>>,
        LS: Stream<Item = zbus::Result<LoopStatus>>,
        SH: Stream<Item = zbus::Result<bool>>,
        ME: Stream<Item = zbus::Result<HashMap<String, OwnedValue>>>,
        CGN: Stream<Item = zbus::Result<bool>>,
        CGP: Stream<Item = zbus::Result<bool>>,
        CPL: Stream<Item = zbus::Result<bool>>,
        CPA: Stream<Item = zbus::Result<bool>>,
    > Drop for PlayerInfoInner<PS, LS, SH, ME, CGN, CGP, CPL, CPA>
{
    fn drop(&mut self) {
        self.playback_status = None;
        self.loop_status = None;
        self.shuffle = None;
        self.metadata = None;
        self.can_go_next = None;
        self.can_go_previous = None;
        self.can_play = None;
        self.can_pause = None;
    }
}

macro_rules! poll_option {
    ($v:expr,$cx:expr) => {
        Pin::new_unchecked($v.as_mut().unwrap_unchecked()).poll_next($cx)
    };
}

impl<
        PS: Stream<Item = zbus::Result<PlaybackStatus>>,
        LS: Stream<Item = zbus::Result<LoopStatus>>,
        SH: Stream<Item = zbus::Result<bool>>,
        ME: Stream<Item = zbus::Result<HashMap<String, OwnedValue>>>,
        CGN: Stream<Item = zbus::Result<bool>>,
        CGP: Stream<Item = zbus::Result<bool>>,
        CPL: Stream<Item = zbus::Result<bool>>,
        CPA: Stream<Item = zbus::Result<bool>>,
    > Stream for PlayerInfoStream<PS, LS, SH, ME, CGN, CGP, CPL, CPA>
{
    type Item = zbus::Result<PlayerInfo>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let (
            info,
            err,
            playback_status,
            loop_status,
            shuffle,
            metadata,
            can_go_next,
            can_go_previous,
            can_play,
            can_pause,
        ) = unsafe {
            let inner = self.get_mut().inner.as_mut().get_unchecked_mut();
            if let Some(e) = inner.err.pop() {
                return Poll::Ready(Some(Err(e)))
            }
            (
                &mut inner.info,
                &mut inner.err,
                poll_option!(inner.playback_status, cx),
                poll_option!(inner.loop_status, cx),
                poll_option!(inner.shuffle, cx),
                poll_option!(inner.metadata, cx),
                poll_option!(inner.can_go_next, cx),
                poll_option!(inner.can_go_previous, cx),
                poll_option!(inner.can_play, cx),
                poll_option!(inner.can_pause, cx),
            )
        };
        if matches!(
            (
                &playback_status,
                &loop_status,
                &shuffle,
                &metadata,
                &can_go_next,
                &can_go_previous,
                &can_play,
                &can_pause
            ),
            (
                Poll::Ready(None),
                Poll::Ready(None),
                Poll::Ready(None),
                Poll::Ready(None),
                Poll::Ready(None),
                Poll::Ready(None),
                Poll::Ready(None),
                Poll::Ready(None)
            )
        ) {
            Poll::Ready(None)
        } else {
            let mut changed = false;
            match playback_status {
                Poll::Ready(Some(Ok(v))) => {
                    changed = true;
                    info.playback_status = v;
                }
                Poll::Ready(Some(Err(e))) => {
                    err.push(e);
                }
                _ => {}
            }
            match loop_status {
                Poll::Ready(Some(Ok(v))) => {
                    changed = true;
                    info.loop_status = Some(v);
                }
                Poll::Ready(Some(Err(e))) => {
                    err.push(e);
                }
                _ => {}
            }
            match shuffle {
                Poll::Ready(Some(Ok(v))) => {
                    changed = true;
                    info.shuffle = Some(v);
                }
                Poll::Ready(Some(Err(e))) => {
                    err.push(e);
                }
                _ => {}
            }
            match metadata {
                Poll::Ready(Some(Ok(v))) => {
                    changed = true;
                    info.metadata = Arc::new(v);
                }
                Poll::Ready(Some(Err(e))) => {
                    err.push(e);
                }
                _ => {}
            }
            match can_go_next {
                Poll::Ready(Some(Ok(v))) => {
                    changed = true;
                    info.can_go_next = v;
                }
                Poll::Ready(Some(Err(e))) => {
                    err.push(e);
                }
                _ => {}
            }
            match can_go_previous {
                Poll::Ready(Some(Ok(v))) => {
                    changed = true;
                    info.can_go_previous = v;
                }
                Poll::Ready(Some(Err(e))) => {
                    err.push(e);
                }
                _ => {}
            }
            match can_play {
                Poll::Ready(Some(Ok(v))) => {
                    changed = true;
                    info.can_play = v;
                }
                Poll::Ready(Some(Err(e))) => {
                    err.push(e);
                }
                _ => {}
            }
            match can_pause {
                Poll::Ready(Some(Ok(v))) => {
                    changed = true;
                    info.can_pause = v;
                }
                Poll::Ready(Some(Err(e))) => {
                    err.push(e);
                }
                _ => {}
            }
            if changed {
                Poll::Ready(Some(Ok(info.clone())))
            } else {
                match err.pop() {
                    Some(e) => Poll::Ready(Some(Err(e))),
                    None => Poll::Pending,
                }
            }
        }
    }
}

pub async fn player_info(
    conn: Connection,
) -> zbus::Result<impl Stream<Item = zbus::Result<PlayerInfo>>> {
    let mut inner_box = Box::pin(PlayerInfoInner {
        conn,
        info: PlayerInfo {
            playback_status: PlaybackStatus::Paused,
            loop_status: None,
            shuffle: None,
            metadata: Arc::new(HashMap::new()),
            can_go_next: false,
            can_go_previous: false,
            can_play: false,
            can_pause: false,
        },
        err: Vec::new(),
        playback_status: None,
        loop_status: None,
        shuffle: None,
        metadata: None,
        can_go_next: None,
        can_go_previous: None,
        can_play: None,
        can_pause: None,
    });
    let inner = unsafe { inner_box.as_mut().get_unchecked_mut() };
    let conn = unsafe { extend_lifetime(&inner.conn) };
    let proxy = PlayerProxy::new(conn).await?;
    inner.playback_status = Some(property_stream(Some(
        proxy.receive_playback_status_changed().await,
    )));
    inner.loop_status = Some(property_stream(Some(
        proxy.receive_loop_status_changed().await,
    )));
    inner.shuffle = Some(property_stream(Some(proxy.receive_shuffle_changed().await)));
    inner.metadata = Some(property_stream(Some(
        proxy.receive_metadata_changed().await,
    )));
    inner.can_go_next = Some(property_stream(Some(
        proxy.receive_can_go_next_changed().await,
    )));
    inner.can_go_previous = Some(property_stream(Some(
        proxy.receive_can_go_previous_changed().await,
    )));
    inner.can_play = Some(property_stream(Some(
        proxy.receive_can_play_changed().await,
    )));
    inner.can_pause = Some(property_stream(Some(
        proxy.receive_can_pause_changed().await,
    )));
    match proxy.playback_status().await {
        Ok(v) => inner.info.playback_status = v,
        Err(e) => inner.err.push(e),
    }
    match proxy.loop_status().await {
        Ok(v) => inner.info.loop_status = Some(v),
        Err(e) => inner.err.push(e),
    }
    match proxy.shuffle().await {
        Ok(v) => inner.info.shuffle = Some(v),
        Err(e) => inner.err.push(e),
    }
    match proxy.metadata().await {
        Ok(v) => inner.info.metadata = Arc::new(v),
        Err(e) => inner.err.push(e),
    }
    match proxy.can_go_next().await {
        Ok(v) => inner.info.can_go_next = v,
        Err(e) => inner.err.push(e),
    }
    match proxy.can_go_previous().await {
        Ok(v) => inner.info.can_go_previous = v,
        Err(e) => inner.err.push(e),
    }
    match proxy.can_play().await {
        Ok(v) => inner.info.can_play = v,
        Err(e) => inner.err.push(e),
    }
    match proxy.can_pause().await {
        Ok(v) => inner.info.can_pause = v,
        Err(e) => inner.err.push(e),
    }
    Ok(PlayerInfoStream { inner: inner_box })
}
