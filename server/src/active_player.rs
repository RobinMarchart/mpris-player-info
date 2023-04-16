use std::{
    future::Future, marker::PhantomPinned, mem::transmute, pin::Pin,
    task::Poll,
};

use crate::proxies::PlayerctldProxy;
use zbus::{
    export::futures_util::Stream,
    Connection, PropertyChanged, PropertyStream,
};

pub struct ActivePlayersStream<F, FUT>
where
    F: Fn(&'static PropertyChanged<'static, Vec<String>>) -> FUT,
    FUT: Future<Output = zbus::Result<Vec<String>>>,
{
    inner: Pin<Box<ActivePlayersInner<F, FUT>>>,
}

struct ActivePlayersInner<F, FUT>
where
    F: Fn(&'static PropertyChanged<'static, Vec<String>>) -> FUT,
    FUT: Future<Output = zbus::Result<Vec<String>>>,
{
    conn: Connection,
    stream: Option<PropertyStream<'static, Vec<String>>>,
    stream_changed: Option<PropertyChanged<'static, Vec<String>>>,
    stream_future: Option<FUT>,
    stream_fun: F,
    first: Option<Vec<String>>,
    _pin: PhantomPinned,
}

impl<
        F: Fn(&'static PropertyChanged<'static, Vec<String>>) -> FUT,
        FUT: Future<Output = zbus::Result<Vec<String>>>,
    > Drop for ActivePlayersInner<F, FUT>
{
    fn drop(&mut self) {
        self.stream_future = None;
        self.stream_changed = None;
        self.stream = None;
    }
}

unsafe fn extend_lifetime<T>(v: &T) -> &'static T {
    transmute(v)
}

pub async fn active_players(
    conn: Connection,
) -> anyhow::Result<impl Stream<Item = zbus::Result<Vec<String>>>> {
    let mut inner = Box::pin(ActivePlayersInner {
        conn,
        _pin: PhantomPinned,
        stream: None,
        stream_future: None,
        stream_changed: None,
        stream_fun: |e| e.get(),
        first: None,
    });
    unsafe {
        let inner = inner.as_mut().get_unchecked_mut();
        let proxy = PlayerctldProxy::new(extend_lifetime(&inner.conn)).await?;
        let player_names = proxy.receive_player_names_changed().await;
        inner.stream = Some(player_names);
        inner.first = Some(proxy.player_names().await?);
    }
    Ok(ActivePlayersStream { inner })
}

impl<
        F: Fn(&'static PropertyChanged<'static, Vec<String>>) -> FUT,
        FUT: Future<Output = zbus::Result<Vec<String>>>,
    > Stream for ActivePlayersStream<F, FUT>
{
    type Item = zbus::Result<Vec<String>>;
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        unsafe {
            let inner = self.get_unchecked_mut().inner.as_mut().get_unchecked_mut();
            match inner.first.take() {
                Some(v) => Poll::Ready(Some(Ok(v))),
                None => match inner.stream_future.as_mut() {
                    Some(f) => match Pin::new_unchecked(f).poll(cx) {
                        Poll::Ready(v) => {
                            inner.stream_future = None;
                            inner.stream_changed = None;
                            Poll::Ready(Some(v))
                        }
                        Poll::Pending => Poll::Pending,
                    },
                    None => {
                        let stream = Pin::new_unchecked(inner.stream.as_mut().unwrap_unchecked());
                        match stream.poll_next(cx) {
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
                            Poll::Ready(None)=>Poll::Ready(None),
                            Poll::Pending=>Poll::Pending
                        }
                    }
                }
            }
        }
    }
}
