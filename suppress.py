
from asyncio import Task,wait, FIRST_COMPLETED
from typing import AsyncGenerator, Tuple, TypeVar

import sdbus

from interfaces import ComGithubRobinmarchartMprisutilsInterface

T=TypeVar("T")

async def with_suppressed(base:AsyncGenerator[T,None])->AsyncGenerator[Tuple[bool,T],None]:
    proxy=ComGithubRobinmarchartMprisutilsInterface.new_proxy(
        "com.github.robinmarchart.mprisutils", "/com/github/robinmarchart/mprisutils",sdbus.sd_bus_open_user()
    )

    state=proxy.properties_changed.catch()
    current_state:bool=await proxy.suppressed
    current_base=None
    base_set=False
    changed=False

    base_task=Task(base.__anext__())
    state_task=Task(state.__anext__())

    try:
        while True:
            (done,_)=await wait([base_task,state_task],return_when=FIRST_COMPLETED)
            if base_task in done:
                base_set=True
                changed=True
                current_base=base_task.result()
                base_task=Task(base.__anext__())
            if state_task in done:
                (interface,changed,_)=state_task.result()
                if interface == "com.github.robinmarchart.mprisutils" and "Suppressed" in changed:
                    current_state=changed["Suppressed"][1]
                    changed=True
                state_task=Task(state.__anext__())
            if base_set and changed:
                changed=False
                yield (current_state,current_base)

    except StopAsyncIteration:
        pass
