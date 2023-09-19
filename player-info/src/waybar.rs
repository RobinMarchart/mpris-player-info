use std::{
    borrow::Cow,
    error::Error,
    future::Future,
    io,
    os::fd::{FromRawFd, IntoRawFd},
    pin::pin,
    sync::Arc,
};

use libsystemd::{
    activation::{receive_descriptors, FileDescriptor},
    daemon::NotifyState,
};
use tokio::{
    io::AsyncWriteExt,
    join,
    net::{UnixListener, UnixStream},
    spawn,
    sync::Mutex,
    try_join,
};
use tracing::{info, warn};

use zbus::{
    export::futures_util::{FutureExt, StreamExt},
    Connection,
};

#[derive(Debug)]
struct Info<'a> {
    text: Option<Cow<'a, str>>,
    tooltip: Option<Cow<'a, str>>,
    class: Option<Cow<'a, str>>,
}

impl<'a> Info<'a> {
    fn serialize(self) -> String {
        let text = self.text.unwrap_or(Cow::Borrowed(""));
        match (self.tooltip, self.class) {
            (None, None) => format!("{{text:\"{text}\"}}\n"),
            (Some(tooltip), None) => format!("{{text:\"{text}\",tooltip:\"{tooltip}\"}}\n"),
            (None, Some(class)) => format!("{{text:\"{text}\",class:\"{class}\"}}\n"),
            (Some(tooltip), Some(class)) => {
                format!("{{text:\"{text}\",class:\"{class}\",tooltip:\"{tooltip}\"}}\n")
            }
        }
    }
}

impl<'a> Default for Info<'a> {
    fn default() -> Self {
        Self {
            text: Default::default(),
            tooltip: Default::default(),
            class: Some("hidden".into()),
        }
    }
}

#[derive(Debug, Default)]
struct Infos<'a> {
    prev_player: Info<'a>,
    next_player: Info<'a>,
    title: Info<'a>,
    play_pause: Info<'a>,
    prev: Info<'a>,
    next: Info<'a>,
}

struct Output {
    inner: Mutex<(String, Vec<Option<UnixStream>>)>,
    listener: UnixListener,
}

impl Output {
    fn new(listener: UnixListener) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(("".to_string(), Vec::new())),
            listener,
        })
    }

    async fn set_message(&self, message: String) {
        let mut inner = self.inner.lock().await;
        for stream in &mut inner.1 {
            if let Err(e) = stream
                .as_mut()
                .expect("stream moved")
                .write_all(message.as_bytes())
                .await
            {
                *stream = None;
                warn!("Error writing message: {e}")
            } else if let Err(e) = stream.as_mut().unwrap().flush().await {
                *stream = None;
                warn!("Error flushing stream: {e}")
            }
        }
        inner.1.retain(Option::is_some);
        inner.0 = message;
    }

    async fn listen(self: Arc<Self>) -> io::Result<()> {
        loop {
            let (mut stream, addr) = self.listener.accept().await?;
            {
                let mut inner = self.inner.lock().await;
                if let Err(e) = stream.write_all(inner.0.as_bytes()).await {
                    warn!("Error writing message: {e}")
                } else if let Err(e) = stream.flush().await {
                    warn!("Error flushing stream: {e}")
                } else {
                    inner.1.push(Some(stream));
                    match addr.as_pathname(){
                        Some(path) => info!("new incoming connection on {}",path.display()),
                        None => info!("new incoming connection on unknown path")
                    }
                }
            }
        }
    }
}

async fn flatten<F, T, E1, E2>(handle: F) -> Result<T, Box<dyn Error>>
where
    F: Future<Output = Result<Result<T, E2>, E1>>,
    E1: Into<Box<dyn Error>>,
    E2: Into<Box<dyn Error>>,
{
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err.into()),
        Err(err) => Err(err.into()),
    }
}

fn to_listener(fd: FileDescriptor) -> io::Result<UnixListener> {
    let listener = unsafe { std::os::unix::net::UnixListener::from_raw_fd(fd.into_raw_fd()) };
    listener.set_nonblocking(true)?;
    UnixListener::from_std(listener)
}

pub async fn waybar(hide: bool) -> Result<(), Box<dyn Error>> {
    let mut sockets = receive_descriptors(false)?;
    if 6 != sockets.len() {
        Err(format!(
            "mismatched number of sockets: expected 6 but actual {}",
            sockets.len()
        ))?;
    }
    let next = Output::new(to_listener(sockets.pop().unwrap())?);
    let prev = Output::new(to_listener(sockets.pop().unwrap())?);
    let play_pause = Output::new(to_listener(sockets.pop().unwrap())?);
    let title = Output::new(to_listener(sockets.pop().unwrap())?);
    let next_player = Output::new(to_listener(sockets.pop().unwrap())?);
    let prev_player = Output::new(to_listener(sockets.pop().unwrap())?);

    let stream =
        mpris_dbus::hide::hidden_active_player_info(&Connection::session().await?, hide).await?;

    libsystemd::daemon::notify(false, &[NotifyState::Ready])?;
    info!("connection established");
    try_join!(
        flatten(spawn(next.clone().listen())),
        flatten(spawn(prev.clone().listen())),
        flatten(spawn(play_pause.clone().listen())),
        flatten(spawn(title.clone().listen())),
        flatten(spawn(next_player.clone().listen())),
        flatten(spawn(prev_player.clone().listen())),
        spawn(async move {
            let mut stream = pin!(stream);
            while let Some(info) = stream.next().await {
                let infos = match info.as_ref() {
                    Some(info) => match info.as_ref() {
                        Some(info) => match info.as_ref() {
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
                                let name = match (artist, title) {
                                    (Some(name), Some(title)) => {
                                        Cow::from(format!("{name} - {title}"))
                                    }
                                    (Some(name), None) => Cow::from(name),
                                    (None, Some(title)) => Cow::from(title),
                                    (None, None) => Cow::from(""),
                                };
                                let (play, play_tooltip) = match info.playback_status {
                                    mpris_dbus::proxies::PlaybackStatus::Playing => ("⏸", "pause"),
                                    mpris_dbus::proxies::PlaybackStatus::Paused => ("▶", "play"),
                                    mpris_dbus::proxies::PlaybackStatus::Stopped => {
                                        ("⬛", "stopped")
                                    }
                                };
                                let (prev_player, next_player) = if names.len() == 1 {
                                    (Info::default(), Info::default())
                                } else {
                                    (
                                        Info {
                                            text: Some("⏶".into()),
                                            tooltip: Some("switch to previous player".into()),
                                            class: None,
                                        },
                                        Info {
                                            text: Some("⏷".into()),
                                            tooltip: Some("switch to next player".into()),
                                            class: None,
                                        },
                                    )
                                };
                                let prev = if info.can_go_previous {
                                    Info {
                                        text: Some("⏮".into()),
                                        tooltip: Some("go to previous".into()),
                                        class: None,
                                    }
                                } else {
                                    Info::default()
                                };
                                let next = if info.can_go_next {
                                    Info {
                                        text: Some("⏭".into()),
                                        tooltip: Some("go to next".into()),
                                        class: None,
                                    }
                                } else {
                                    Info::default()
                                };
                                Infos {
                                    prev_player,
                                    next_player,
                                    title: Info {
                                        text: Some(name.clone()),
                                        tooltip: Some(name),
                                        class: None,
                                    },
                                    play_pause: Info {
                                        text: Some(play.into()),
                                        tooltip: Some(play_tooltip.into()),
                                        class: None,
                                    },
                                    prev,
                                    next,
                                }
                            }
                            Err(e) => {
                                warn!("{e}");
                                Infos {
                                    title: Info {
                                        text: Some(format!("{e}").into()),
                                        tooltip: Some(format!("{e}").into()),
                                        class: None,
                                    },
                                    ..Default::default()
                                }
                            }
                        },
                        None => Infos {
                            title: Info {
                                text: Some("no player".into()),
                                tooltip: None,
                                class: None,
                            },
                            ..Default::default()
                        },
                    },
                    None => Infos {
                        ..Default::default()
                    },
                };
                join!(
                    next.set_message(infos.next.serialize()),
                    prev.set_message(infos.prev.serialize()),
                    title.set_message(infos.title.serialize()),
                    play_pause.set_message(infos.play_pause.serialize()),
                    next_player.set_message(infos.next_player.serialize()),
                    prev_player.set_message(infos.prev_player.serialize())
                );
            }
        })
        .map(|e| e.map_err(Box::<dyn Error>::from))
    )?;
    Ok(())
}
