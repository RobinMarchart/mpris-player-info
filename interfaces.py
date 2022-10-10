from typing import Any, Dict, List, Tuple

from sdbus import (
    DbusInterfaceCommonAsync,
    dbus_method_async,
    dbus_property_async,
    dbus_signal_async,
)


class OrgMprisMediaPlayer2Interface(
    DbusInterfaceCommonAsync,
    interface_name="org.mpris.MediaPlayer2",
):
    @dbus_method_async(method_name="raise")
    async def raise_player(
        self,
    ) -> None:
        raise NotImplementedError

    @dbus_method_async()
    async def quit(
        self,
    ) -> None:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def can_quit(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def fullscreen(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def can_set_fullscreen(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def can_raise(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def has_track_list(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="s",
    )
    def identity(self) -> str:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="s",
    )
    def desktop_entry(self) -> str:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="as",
    )
    def supported_uri_schemes(self) -> List[str]:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="as",
    )
    def supported_mime_types(self) -> List[str]:
        raise NotImplementedError


class ComGithubRobinmarchartMprisutilsInterface(
    DbusInterfaceCommonAsync,
    interface_name="com.github.robinmarchart.mprisutils",
):
    @dbus_method_async(
        result_signature="b",
    )
    async def toggle(
        self,
    ) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def suppressed(self) -> bool:
        raise NotImplementedError


class ComGithubAltdesktopPlayerctldInterface(
    DbusInterfaceCommonAsync,
    interface_name="com.github.altdesktop.playerctld",
):
    @dbus_method_async(
        result_signature="s",
    )
    async def shift(
        self,
    ) -> str:
        raise NotImplementedError

    @dbus_method_async(
        result_signature="s",
    )
    async def unshift(
        self,
    ) -> str:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="as",
    )
    def player_names(self) -> List[str]:
        raise NotImplementedError

    @dbus_signal_async(
        signal_signature="s",
    )
    def active_player_change_begin(self) -> str:
        raise NotImplementedError

    @dbus_signal_async(
        signal_signature="s",
    )
    def active_player_change_end(self) -> str:
        raise NotImplementedError


class OrgMprisMediaPlayer2PlayerInterface(
    DbusInterfaceCommonAsync,
    interface_name="org.mpris.MediaPlayer2.Player",
):
    @dbus_method_async()
    async def next(
        self,
    ) -> None:
        raise NotImplementedError

    @dbus_method_async()
    async def previous(
        self,
    ) -> None:
        raise NotImplementedError

    @dbus_method_async()
    async def pause(
        self,
    ) -> None:
        raise NotImplementedError

    @dbus_method_async()
    async def play_pause(
        self,
    ) -> None:
        raise NotImplementedError

    @dbus_method_async()
    async def stop(
        self,
    ) -> None:
        raise NotImplementedError

    @dbus_method_async()
    async def play(
        self,
    ) -> None:
        raise NotImplementedError

    @dbus_method_async(
        input_signature="x",
    )
    async def seek(
        self,
        offset: int,
    ) -> None:
        raise NotImplementedError

    @dbus_method_async(
        input_signature="ox",
    )
    async def set_position(
        self,
        track_id: str,
        offset: int,
    ) -> None:
        raise NotImplementedError

    @dbus_method_async(
        input_signature="s",
    )
    async def open_uri(
        self,
        uri: str,
    ) -> None:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="s",
    )
    def playback_status(self) -> str:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="s",
    )
    def loop_status(self) -> str:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="d",
    )
    def rate(self) -> float:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def shuffle(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="a{sv}",
    )
    def metadata(self) -> Dict[str, Tuple[str, Any]]:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="d",
    )
    def volume(self) -> float:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="x",
    )
    def position(self) -> int:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="d",
    )
    def minimum_rate(self) -> float:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="d",
    )
    def maximum_rate(self) -> float:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def can_go_next(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def can_go_previous(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def can_play(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def can_pause(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def can_seek(self) -> bool:
        raise NotImplementedError

    @dbus_property_async(
        property_signature="b",
    )
    def can_control(self) -> bool:
        raise NotImplementedError

    @dbus_signal_async(
        signal_signature="x",
    )
    def seeked(self) -> int:
        raise NotImplementedError
