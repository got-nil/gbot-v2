"""

    The bot is the actual connected gmod instance. It supplies an identifier and then
    can be used to send / receive operation data between the game instance and this server.
    These are then passed on to the requesting client.

    It is connected by websocket, and communicates in binary.

"""

from typing import Dict

from identifiable_websocket import IdentifiableWebsocket
from operations import OperationReply, PendingOperation


class Bot(IdentifiableWebsocket):

    pending_operations: Dict[str, PendingOperation]

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.pending_operations = {}

    async def recv(self) -> None:

        async for msg in self.websocket:

            # All messages should be sent back as packed binary bytes.
            if type(msg) is not bytes:
                self.log("Received an invalid non-bytes message!")
                continue

            # Decode reply.
            reply = OperationReply.from_bytes(msg)
            if reply is None:
                self.log("Received an invalid operation reply! Could not decode.")
                continue

            # Find the pending operation for the received reply_id.
            pending_operation = self.pending_operations.get(reply.reply_id)
            if pending_operation is None:
                self.log(f"Received a reply for an unknown / non-pending operation '{reply.reply_id}'!")
                continue

            # Handle the constructed reply then remove it from the pending operations set.
            await self.handle_reply(pending_operation, reply)
            self.pending_operations.pop(reply.reply_id)

    async def handle_reply(self, pending: PendingOperation, reply: OperationReply) -> None:

        # Find the client that actually requested the operation.
        client = self.container.clients.get(pending.client_id)
        if client is None:
            self.log(f"Received a reply '{reply.reply_id}' for non-existent client '{pending.client_id}'!")
            return

        # Allow the calling client to handle the reply itself.
        self.log(f"Passing reply '{reply.reply_id}' to client '{client.identifier}'!")
        await client.handle_reply(reply)

    async def send_operation(self, op: PendingOperation) -> bool:

        # Make sure we're not closed and somehow still referenced.
        if not await self.check_closed():
            self.log("Could not send operation as socket is closed!")
            return False

        # Actually send the operation as bytes.
        self.log("Sending operation: " + op.operation.__repr__())
        await self.websocket.send(
            op.operation.to_bytes()
        )
        self.pending_operations[op.operation.reply_id] = op
        return True
