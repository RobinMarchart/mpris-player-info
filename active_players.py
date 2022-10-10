from typing import AsyncGenerator
from interfaces import ComGithubAltdesktopPlayerctldInterface


async def active_players(
) -> AsyncGenerator[list[str], None]:
    playerctld_proxy = ComGithubAltdesktopPlayerctldInterface.new_proxy(
        "org.mpris.MediaPlayer2.playerctld", "/org/mpris/MediaPlayer2"
    )
    yield await playerctld_proxy.player_names.get_async()
    async for (interface, changed, invalidated) in playerctld_proxy.properties_changed:
        if interface == "com.github.altdesktop.playerctld" and "PlayerNames" in changed:
            player_names = changed["PlayerNames"][1]
            yield player_names
        elif (
            interface == "com.github.altdesktop.playerctld"
            and "PlayerNames" in invalidated
        ):
            yield []
