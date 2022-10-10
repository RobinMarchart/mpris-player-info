from typing import Tuple

from sdbus.sd_bus_internals import SdBus
from interfaces import OrgMprisMediaPlayer2PlayerInterface
from asyncio import gather
from collections.abc import AsyncGenerator
from sys import stderr


class State:
    url: str | None
    artist: str | None
    title: str | None
    playing: bool

    def __init__(
        self, url: str | None, artist: str | None, title: str | None, playing: bool
    ) -> None:
        self.url = url
        self.artist = artist
        self.title = title
        self.playing = playing

    def __repr__(self) -> str:
        return f"State(url={self.url},artist={self.artist},title={self.title},playing={self.playing})"


async def player_state(player: str) -> AsyncGenerator[State, None]:
    proxy = OrgMprisMediaPlayer2PlayerInterface.new_proxy(
        player, "/org/mpris/MediaPlayer2"
    )
    res = await gather(proxy.metadata.get_async(), proxy.playback_status.get_async())
    metadata_tmp: dict[str, Tuple[str, str]] = res[0]
    playing_tmp: str = res[1]
    url_tmp = (
        metadata_tmp["mpris:artUrl"][1] if "mpris:artUrl" in metadata_tmp else None
    )
    artist = metadata_tmp["xesam:artist"][1] if "xesam:artist" in metadata_tmp else None
    title = metadata_tmp["xesam:title"][1] if "xesam:title" in metadata_tmp else None
    if (not artist == None) and len(artist) == 0 or not isinstance(artist,str):
        artist = None
    if (not title == None) and len(title) == 0:
        title = None
    if (not url_tmp == None) and len(url_tmp) == 0:
        url = None
    yield State(
        url_tmp,
        artist,
        title,
        True if playing_tmp == "Playing" else False,
    )
    metadata: dict[str, Tuple[str, str]] | None = None
    playing: str | None = None
    async for (interface, changed, invalidated) in proxy.properties_changed:
        if interface == "org.mpris.MediaPlayer2.Player":
            found = False
            if "Metadata" in changed:
                metadata = changed["Metadata"][1]
                found = True
            if "PlaybackStatus" in changed:
                playing = changed["PlaybackStatus"][1]
                found = True
            if "Metadata" in invalidated:
                metadata = None
                found = True
            if "PlaybackStatus" in invalidated:
                playing = None
                found = True
            if found:
                if metadata == None:
                    metadata = await proxy.metadata.get_async()
                if playing == None:
                    playing = await proxy.playback_status.get_async()
                artist = None
                title = None
                url = None
                playing_procesed = None
                if metadata == None or playing == None:
                    print("property empty", file=stderr)
                else:
                    if "xesam:artist" in metadata:
                        artist = metadata["xesam:artist"][1]
                        if len(artist) == 0 or not isinstance(artist,str):
                            artist = None

                    if "xesam:title" in metadata:
                        title = metadata["xesam:title"][1]
                        if len(title) == 0 or not isinstance(title,str):
                            title = None
                    if "mpris:artUrl" in metadata:
                        url = metadata["mpris:artUrl"][1]
                    playing_procesed = (
                        True
                        if playing == "Playing"
                        else False
                        if playing == "Paused"
                        else None
                    )
                    if playing_procesed == None:
                        print(f"unknown status {playing}", file=stderr)
                    else:
                        yield State(url, artist, title, playing_procesed)
