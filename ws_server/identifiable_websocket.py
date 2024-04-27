from typing import TYPE_CHECKING
from sanic import Websocket
from websockets.protocol import State

if TYPE_CHECKING:
    from container import Container


class IdentifiableWebsocket:

    container: "Container"
    identifier: str
    websocket: Websocket
    closed: bool

    def __init__(self, container: "Container", identifier: str, websocket: Websocket):
        self.container = container
        self.identifier = identifier
        self.websocket = websocket
        self.closed = False

    def log(self, string: str) -> None:
        print(self.identifier, " -> ", string)

    async def check_closed(self) -> bool:

        if self.websocket.ws_proto.state in (State.CLOSED, State.CLOSING):
            await self.close()
            return False

        return True

    async def close(self) -> None:

        if self.closed:
            return

        await self.websocket.close()
        self.closed = True

        # Remove the current object from container.
        await self.container.remove_obj(self)

    async def recv(self) -> None:
        raise NotImplemented()
