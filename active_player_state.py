from asyncio import wait,FIRST_COMPLETED,Task
from typing import AsyncGenerator, Tuple

from active_players import active_players
from player_state import State, player_state

async def active_player_state()->AsyncGenerator[Tuple[str,State]|None,None]:
    active=active_players()
    player=None
    player_name:str|None=None
    current_player_state:State|None=None
    active_task=Task(active.__anext__())
    while True:
        if player==None:
            yield None
            players=await active_task
            active_task=Task(active.__anext__())
            player_name=None if len(players)==0 else players[0]
            player=None if player_name==None else player_state(player_name)
        else:
            if player_name!=None and current_player_state!=None:
                yield (player_name,current_player_state)
            player_task=Task(player.__anext__())
            (done,_)=await wait([player_task,active_task],return_when=FIRST_COMPLETED)
            if player_task in done:
                try:
                    current_player_state= player_task.result()
                except StopAsyncIteration:
                    player=None
            else:
                player_task.cancel()
                players=active_task.result()
                active_task=Task(active.__anext__())
                player_name=None if len(players)==0 else players[0]
                player=None if player_name==None else player_state(player_name)
