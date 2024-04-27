from sanic import Sanic, Request, Websocket
from typing import Union, Tuple, Type, Literal

from client import Client
from bot import Bot
from container import Container


class Main:

    app: Sanic
    container: Container

    def __init__(self, app: Sanic):
        self.app = app
        self.container = Container()

        app.add_websocket_route(self.route_ws_bot, "/ws/bot")
        app.add_websocket_route(self.route_ws_client, "/ws/client")

    @staticmethod
    def validate_request(request: Request) -> Tuple[bool, str]:

        # Make sure there is an identifier.
        # identifier = request.headers.get("X-Id")
        identifier = "client00000000000"
        if identifier is None:
            return False, "Missing required identifier!"

        # The identifier must be at least 8 characters.
        if len(identifier) < 8:
            return False, "Invalid identifier!"

        # TODO: Authorization.
        return True, identifier

    async def generic_websocket_init(self, target_obj: Union[Type[Client], Type[Bot]], request: Request, ws: Websocket) -> None:

        # Validate the request.
        success, out = self.validate_request(request)
        if not success:
            await ws.close(4001, out)
            return

        # Create the object and add it to container by its identifier.
        obj = target_obj(self.container, out, ws)
        if not await self.container.add_obj(obj):

            # Failed to add to container, probably identifier conflict.
            await ws.close(4002, "Failed to add to container, possible identifier conflict!")
            return

        # Start receiver loop.
        await obj.recv()

    async def route_ws_bot(self, request: Request, ws: Websocket) -> None:
        return await self.generic_websocket_init(Bot, request, ws)

    async def route_ws_client(self, request: Request, ws: Websocket) -> None:
        return await self.generic_websocket_init(Client, request, ws)

    def run(self) -> None:
        self.app.run(
            debug=True,
            access_log=True
        )


MAIN = Main(Sanic("gbot-websocket"))
if __name__ == "__main__":
    MAIN.run()
