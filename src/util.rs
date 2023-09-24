#[cfg(any(feature = "hide", feature = "playerctld", feature = "player_info"))]
use std::future;

#[cfg(any(feature = "hide", feature = "player_info"))]
use std::{
    pin::Pin,
    task::Poll::{self, Pending, Ready},
};

#[cfg(feature = "player_info")]
use std::mem::take;

#[cfg(feature = "active_player_info")]
use std::{collections::HashSet, sync::Mutex};

#[cfg(any(feature = "player_info", feature = "hide", feature = "playerctld"))]
use tracing::{event, Level, Span};

#[cfg(any(feature = "hide", feature = "playerctld", feature = "player_info"))]
use zbus::export::futures_util::{
    stream::{self, Chain, Once},
    Stream, StreamExt,
};

#[cfg(feature = "player_info")]
pub struct FoldMap<B, S: Stream, T, F: FnMut(S::Item, B) -> (T, B)> {
    val: Option<B>,
    fun: F,
    stream: S,
}

#[cfg(any(feature = "player_info", feature = "hide", feature = "playerctld"))]
pub trait StreamExt2: Stream {
    #[cfg(feature = "player_info")]
    fn fold_map<B, T, F: FnMut(Self::Item, B) -> (T, B)>(
        self,
        init: B,
        map: F,
    ) -> FoldMap<B, Self, T, F>
    where
        Self: Sized,
    {
        FoldMap {
            val: Some(init),
            fun: map,
            stream: self,
        }
    }

    #[cfg(feature = "active_player_info")]
    fn flatten_newest(self) -> FlattenNewest<Self, Self::Item>
    where
        Self: Sized,
        Self::Item: Stream,
    {
        FlattenNewest { s1: self, s2: None }
    }

    fn with_initial_value(self, val: Self::Item) -> Chain<Once<future::Ready<Self::Item>>, Self>
    where
        Self: Sized,
    {
        stream::once(future::ready(val)).chain(self)
    }

    fn instrument_stream(self, span: Span) -> InstrumentedStream<Self>
    where
        Self: Sized,
    {
        InstrumentedStream { s: self, span }
    }

    fn filter_no_change(self) -> FilterNoChange<Self>
    where
        Self: Sized,
        Self::Item: Clone + PartialEq,
    {
        FilterNoChange {
            stream: self,
            last: None,
        }
    }
}

#[cfg(any(feature = "player_info", feature = "hide", feature = "playerctld"))]
impl<S: Stream> StreamExt2 for S {}

#[cfg(feature = "player_info")]
impl<B, S: Stream, T, F: FnMut(S::Item, B) -> (T, B)> Stream for FoldMap<B, S, T, F> {
    type Item = T;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let (v, f, s) = unsafe {
            let s = self.get_unchecked_mut();
            (&mut s.val, &mut s.fun, Pin::new_unchecked(&mut s.stream))
        };
        match s.poll_next(cx) {
            Pending => Pending,
            Ready(None) => Ready(None),
            Ready(Some(next)) => {
                let val = take(v).expect("missing internal state. Did the map function panic?");
                let (next, val) = f(next, val);
                *v = Some(val);
                Ready(Some(next))
            }
        }
    }
}

#[cfg(feature = "active_player_info")]
static STRING_STORE: Mutex<Option<HashSet<Box<str>>>> = Mutex::new(None);

#[cfg(feature = "active_player_info")]
pub fn string_to_static(str: String) -> &'static str {
    let mut store = STRING_STORE.lock().unwrap();
    if store.is_none() {
        store.replace(HashSet::new());
    }
    match store.as_mut() {
        Some(set) => {
            let str = str.into_boxed_str();
            let ptr = match set.get(&str) {
                Some(str) => str.as_ref() as *const str,
                None => {
                    let ptr: *const str = str.as_ref();
                    assert!(set.insert(str));
                    ptr
                }
            };
            //the box will never be removed, therefore the reference to it's content is static.
            unsafe { &*ptr }
        }
        None => unreachable!(),
    }
}

#[cfg(feature = "active_player_info")]
pub struct FlattenNewest<S1, S2>
where
    S1: Stream<Item = S2>,
    S2: Stream,
{
    s1: S1,
    s2: Option<S2>,
}

#[cfg(feature = "active_player_info")]
impl<S1, S2> Stream for FlattenNewest<S1, S2>
where
    S1: Stream<Item = S2>,
    S2: Stream,
{
    type Item = S2::Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        unsafe {
            let s = self.get_unchecked_mut();
            loop {
                match Pin::new_unchecked(&mut s.s1).poll_next(cx) {
                    Ready(None) => {
                        return Ready(None);
                    }
                    Ready(Some(s2)) => {
                        s.s2 = Some(s2);
                    }
                    Pending => {}
                }
                match s.s2.as_mut() {
                    Some(s2) => match Pin::new_unchecked(s2).poll_next(cx) {
                        Ready(None) => {
                            s.s2 = None;
                        }
                        r => return r,
                    },
                    None => return Pending,
                };
            }
        }
    }
}

#[cfg(feature = "hide")]
pub struct PollBoth<S1: Stream, S2: Stream> {
    s1: S1,
    s2: S2,
    v1: Option<S1::Item>,
    v2: Option<S2::Item>,
}

#[cfg(feature = "hide")]
impl<S1: Stream, S2: Stream> Stream for PollBoth<S1, S2>
where
    S1::Item: Clone,
    S2::Item: Clone,
{
    type Item = (Option<S1::Item>, Option<S2::Item>);

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let (s1, s2, v1, v2) = unsafe {
            let s = self.get_unchecked_mut();
            (
                Pin::new_unchecked(&mut s.s1),
                Pin::new_unchecked(&mut s.s2),
                &mut s.v1,
                &mut s.v2,
            )
        };
        match (s1.poll_next(cx), s2.poll_next(cx)) {
            (Ready(None), _) | (_, Ready(None)) => Ready(None),
            (Pending, Pending) => Pending,
            (Ready(val1), Ready(val2)) => {
                *v1 = val1.clone();
                *v2 = val2.clone();
                Ready(Some((val1, val2)))
            }
            (Ready(val1), Pending) => {
                *v1 = val1.clone();
                Ready(Some((val1, v2.clone())))
            }
            (Pending, Ready(val2)) => {
                *v2 = val2.clone();
                Ready(Some((v1.clone(), val2)))
            }
        }
    }
}

#[cfg(feature = "hide")]
pub fn poll_both<S1: Stream, S2: Stream>(s1: S1, s2: S2) -> PollBoth<S1, S2>
where
    S1::Item: Clone,
    S2::Item: Clone,
{
    PollBoth {
        s1,
        s2,
        v1: None,
        v2: None,
    }
}

#[cfg(any(feature = "player_info", feature = "hide", feature = "playerctld"))]
pub struct InstrumentedStream<S: Stream> {
    s: S,
    span: Span,
}

#[cfg(any(feature = "player_info", feature = "hide", feature = "playerctld"))]
impl<S: Stream> Stream for InstrumentedStream<S> {
    type Item = S::Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let (s, span) = unsafe {
            let s = self.get_unchecked_mut();
            (Pin::new_unchecked(&mut s.s), &mut s.span)
        };
        span.in_scope(|| s.poll_next(cx))
    }
}

#[cfg(any(feature = "player_info", feature = "hide", feature = "playerctld"))]
pub trait ResultExt {
    fn trace_err(self) -> Self;
    fn trace_err_span(self, span: &Span) -> Self;
}

#[cfg(any(feature = "player_info", feature = "hide", feature = "playerctld"))]
impl<T, E: std::error::Error> ResultExt for Result<T, E> {
    fn trace_err(self) -> Self {
        match self {
            Err(e) => {
                event!(Level::ERROR,error = %e);
                Err(e)
            }
            ok => ok,
        }
    }

    fn trace_err_span(self, span: &Span) -> Self {
        match self {
            Err(e) => {
                span.in_scope(|| event!(Level::ERROR,error = %e));
                Err(e)
            }
            ok => ok,
        }
    }
}

#[cfg(any(feature = "player_info", feature = "hide", feature = "playerctld"))]
pub struct FilterNoChange<S: Stream> {
    stream: S,
    last: Option<S::Item>,
}

#[cfg(any(feature = "player_info", feature = "hide", feature = "playerctld"))]
impl<S: Stream> Stream for FilterNoChange<S>
where
    S::Item: Clone + PartialEq,
{
    type Item = S::Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let (mut stream, last) = unsafe {
            let s = self.get_unchecked_mut();
            (Pin::new_unchecked(&mut s.stream), &mut s.last)
        };
        loop {
            break match (stream.as_mut().poll_next(cx), last.as_ref()) {
                (Ready(Some(v)), Some(cmp)) => {
                    if &v == cmp {
                        continue;
                    } else {
                        *last = Some(v.clone());
                        Ready(Some(v))
                    }
                }
                (Ready(Some(v)), None) => {
                    *last = Some(v.clone());
                    Ready(Some(v))
                }
                (Ready(None), _) => {
                    *last = None;
                    Ready(None)
                }
                (Pending, _) => Pending,
            };
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        future::{poll_fn, Future},
        pin::{pin, Pin},
        task::Poll,
    };

    use super::StreamExt2;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use zbus::export::futures_util::stream::{self, StreamExt};

    macro_rules! assert_pending {
        ($f:expr, $s:literal) => {
            let mut f = $f;
            poll_fn(|cx| {
                assert!(
                    matches!(Future::poll(Pin::as_mut(&mut f), cx), Poll::Pending),
                    $s
                );
                Poll::Ready(())
            })
            .await
        };
    }

    #[tokio::test]
    async fn poll_both() {
        let (in1, stream1) = mpsc::channel::<u8>(4);
        let (in2, stream2) = mpsc::channel::<i8>(4);
        let mut s = pin!(super::poll_both(
            ReceiverStream::new(stream1),
            ReceiverStream::new(stream2)
        ));
        assert_pending!(pin!(s.next()),"both streams empty");
        in1.send(255).await.unwrap();
        assert_eq!(Some((Some(255),None)),s.next().await,"streams partially filled");
        assert_pending!(pin!(s.next()),"already polled");
        in2.send(-1).await.unwrap();
        assert_eq!(Some((Some(255),Some(-1))),s.next().await,"streams filled");
        assert_pending!(pin!(s.next()),"already polled");
        in1.send(244).await.unwrap();
        assert_eq!(Some((Some(244),Some(-1))),s.next().await,"value changed 1");
        assert_pending!(pin!(s.next()),"already polled");
        in2.send(-2).await.unwrap();
        assert_eq!(Some((Some(244),Some(-2))),s.next().await,"value changed 2");
        assert_pending!(pin!(s.next()),"already polled");
        drop(in1);
        assert_eq!(None,s.next().await,"stream 1 closed");


        let (in1, stream1) = mpsc::channel::<u8>(4);
        let (in2, stream2) = mpsc::channel::<i8>(4);
        let mut s = pin!(super::poll_both(
            ReceiverStream::new(stream1),
            ReceiverStream::new(stream2)
        ));

        assert_pending!(pin!(s.next()),"both streams empty");
        in2.send(-1).await.unwrap();
        assert_eq!(Some((None,Some(-1))),s.next().await,"streams partially filled");
        assert_pending!(pin!(s.next()),"already polled");
        in1.send(255).await.unwrap();
        assert_eq!(Some((Some(255),Some(-1))),s.next().await,"streams filled");
        assert_pending!(pin!(s.next()),"already polled");
        drop(in2);
        assert_eq!(None,s.next().await,"stream 2 closed");
    }

    #[tokio::test]
    async fn filter_no_change() {
        let mut stream = pin!(stream::iter([1u8, 1]).filter_no_change());
        assert_eq!(Some(1u8), stream.next().await, "first value missing");
        assert_eq!(
            None,
            stream.next().await,
            "second value should have been dropped"
        );
    }
}
