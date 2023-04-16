#!/usr/bin/env python3

from asyncio import run

from active_player_state import active_player_state
import argparse
from sys import stdout

from active_players import active_players
from interfaces import ComGithubRobinmarchartMprisutilsInterface
from player_state import player_state
from suppress import with_suppressed

async def print_status(max_len: int,icon_font:int):
    async for (suppressed,state) in with_suppressed(active_player_state()):
        if suppressed:
            print(f"%{{T{icon_font}}}%{{A1:mpris-helper-toggle:}}🐧%{{A}}%{{T-}}")
            stdout.flush()
        elif state == None:
            print(f"%{{T{icon_font}}}%{{A1:mpris-helper-toggle:}}🐧%{{A}}%{{T-}} No Player")
            stdout.flush()
        else:
            state = state[1]
            if (
                state.title != None
                and state.artist != None
                and state.title.startswith(state.artist)
            ):
                state.artist = None
            string = (
                f"{state.artist} - {state.title}"
                if state.title != None and state.artist != None
                else (
                    state.artist
                    if state.artist != None
                    else (state.title if state.title != None else "")
                )
            )
            if len(string) > max_len:
                string = f"{string[:max_len-1]}…"
            print(
                f"%{{T{icon_font}}}%{{A1:mpris-helper-toggle:}}🐧%{{A}}%{{T-}} %{{T{icon_font}}}%{{A1:playerctld shift:}}⏶%{{A}}%{{T-}} %{{T{icon_font}}}%{{A1:playerctld unshift:}}⏷%{{A}}%{{T-}} "
                f"{string} %{{T{icon_font}}}%{{A1:playerctl play-pause:}}{ '⏸' if state.playing else '▶'}"
                f"%{{A}}%{{T-}} %{{T{icon_font}}}%{{A1:playerctl previous:}}⏮%{{A}}%{{T-}} %{{T{icon_font}}}%{{A1:playerctl next:}}⏭%{{A}}%{{T-}}"
            )
            stdout.flush()
    print("playerctld exited")


async def print_player(player: str, max_len: int,icon_font:int):
    async for state in player_state(player):
        if (
            state.title != None
            and state.artist != None
            and state.title.startswith(state.artist)
        ):
            state.artist = None
        string = (
            f"{state.artist} - {state.title}"
            if state.title != None and state.artist != None
            else (
                state.artist
                if state.artist != None
                else (state.title if state.title != None else "")
            )
        )
        if len(string) > max_len:
            string = f"{string[:max_len-1]}…"
        print(
            f"%{{T{icon_font}}}%{{A1:playerctld shift:}}⏶%{{A}}%{{T-}} %{{T{icon_font}}}%{{A1:playerctld unshift:}}⏷%{{A}}%{{T-}} 🐧 "
            f"{string} %{{T{icon_font}}}%{{A1:playerctl play-pause:}}"
            f"{ '⏸' if state.playing else '▶'}"
            f"%{{A}} %{{T{icon_font}}}%{{A1:playerctl previous:}}⏮%{{A}} %{{A1:playerctl next:}}⏭%{{A}}%{{T-}}"
        )
        stdout.flush()


async def print_active_player():
    async for player in active_players():
        print(player)
        stdout.flush()


async def toggle_daemon_value():
    proxy=ComGithubRobinmarchartMprisutilsInterface.new_proxy(
        "com.github.robinmarchart.mprisutils", "/com/github/robinmarchart/mprisutils"
    )
    await proxy.toggle()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="mpris-helper", description="Some tools for working with mpris players"
    )
    sub_parsers = parser.add_subparsers(required=True)

    info = sub_parsers.add_parser("info")
    info.set_defaults(name="info")
    info.add_argument("-l", "--length", type=int, default=50)
    info.add_argument("-i", "--icon-font", type=int, default=2,help="1 based index of the icon font")

    info = sub_parsers.add_parser("info-player")
    info.set_defaults(name="info-player")
    info.add_argument("-l", "--length", type=int, default=50)
    info.add_argument("-i", "--icon-font", type=int, default=2,help="1 based index of the icon font")
    info.add_argument("-p", "--player", default="org.mpris.MediaPlayer2.playerctld")

    toggle = sub_parsers.add_parser("toggle")
    toggle.set_defaults(name="toggle")

    active_player = sub_parsers.add_parser("active-player")
    active_player.set_defaults(name="active-player")

    results = parser.parse_args()
    if results.name == "info":
        run(print_status(results.length,results.icon_font))
    elif results.name == "info-player":
        run(print_player(results.player, results.length,results.icon_font))
    elif results.name == "active-player":
        run(print_active_player())
    elif results.name == "toggle":
        run(toggle_daemon_value())
    else:
        print("unknown command")
        exit(1)
