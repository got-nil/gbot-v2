from typing import Dict, Union, Optional, TypeVar

from identifiable_websocket import IdentifiableWebsocket
from client import Client
from bot import Bot

OBJ_TYPE_TARGETS = {
    Client: "clients",
    Bot: "bots"
}
T = TypeVar("T", bound=IdentifiableWebsocket)


class Container:

    clients: Dict[str, Client]
    bots: Dict[str, Bot]

    def __init__(self):
        self.clients = {}
        self.bots = {}

    # Add an object to a target dict by identifier.
    # Do not allow an existing identifier to be replaced.
    async def add_obj(self, obj: T) -> bool:

        # Get target dict from obj type.
        target = OBJ_TYPE_TARGETS.get(type(obj))
        if target is None:
            return False

        # Only allow object replacement if the previous is closed.
        current_obj: Optional[T] = getattr(self, target).get(obj.identifier)
        if current_obj is not None:

            # Reject if it's not yet closed.
            if await current_obj.check_closed():
                return False

        # Finally, add the new object.
        getattr(self, target)[obj.identifier] = obj
        return True

    async def remove_obj(self, obj: T) -> bool:

        # Get target dict from obj type.
        target = OBJ_TYPE_TARGETS.get(type(obj))
        if target is None:
            return False

        # Make sure there actually is an object in this target with the given id.
        current_target = getattr(self, target)
        if obj.identifier not in current_target:
            return False

        # Make sure the connection has been closed since we're about to lose its reference.
        await current_target[obj.identifier].close()

        # Delete the identifier from target.
        getattr(self, target).pop(obj.identifier)
        return True
