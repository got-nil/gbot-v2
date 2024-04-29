"""

    The client is an external entity that is sending operations to requested bot(s).
    It is connected by websocket, and communicates in JSON.

"""

import msgspec
from typing import Annotated, Any, List, Tuple, Optional

from identifiable_websocket import IdentifiableWebsocket
from operations import OperationType, OperationReply, Operation, PendingOperation


class RequestMessage(msgspec.Struct):
    id: Annotated[str, msgspec.Meta(min_length=8, max_length=28)]  # Reply ID.
    op: OperationType  # Operation type.
    target: Annotated[str, msgspec.Meta(min_length=8, max_length=28)] | None = None  # Target bot ID.
    payloads: List[str] | None = None  # Payloads to send.


class ReplyMessage(msgspec.Struct):
    id: str | None  # Only optional if we fail when decoding.
    success: bool
    output: Any = None
    error_message: str | None = None


class Client(IdentifiableWebsocket):

    # Handle an operation that shouldn't be sent to a bot.
    async def handle_server_operation(self, request: RequestMessage) -> Tuple[bool, Optional[str]]:

        match request.op:
            case OperationType.ListBots:

                # Reply with all existing bot identifiers from container keys.
                await self.send_reply_message(
                    ReplyMessage(
                        id=request.id,
                        success=True,
                        output=list(self.container.bots.keys())
                    )
                )
                return True, None

            case _:
                return False, "Invalid/Unknown server operation!"

    # Dispatch / Handle an operation request message.
    async def dispatch_operation(self, request: RequestMessage) -> Tuple[bool, Optional[str]]:

        # If it's not a bot enum, handle differently.
        is_bot_enum, requires_payload = request.op.meta()
        if not is_bot_enum:
            return await self.handle_server_operation(request)

        # If it's a bot operation, there must be a target.
        if request.target is None:
            return False, "Bot operations must include a target identifier!"

        # If a payload is required, make sure it exists.
        if requires_payload and request.payloads is None:
            return False, "This operation requires payload(s)!"

        # Find the target bot from request "target".
        bot = self.container.bots.get(request.target)
        if bot is None:
            return False, f"Could not find target bot '{request.target}'!"

        # Create a pending operation from the request message and send it to the bot.
        pending = PendingOperation(
            client_id=self.identifier,
            operation=Operation(
                reply_id=request.id,
                type=request.op,
                payloads=request.payloads
            )
        )
        success = await bot.send_operation(pending)
        return success, None if success else "Failed to send operation to target bot!"

    async def recv(self) -> None:

        async for msg in self.websocket:

            # All messages should be sent as a string (json).
            if type(msg) is not str:
                self.log("Received an invalid non-string message!")
                continue

            # Deserialize message into a RequestMessage struct.
            try:
                message = msgspec.json.decode(msg, type=RequestMessage)
            except msgspec.MsgspecError as e:
                self.log("Failed to decode RequestMessage from client: " + e.args[0])

                # Send an error back without an identifier since
                # we weren't able to decode the message to get it.
                await self.send_reply_message(
                    ReplyMessage(
                        id=None,
                        success=False,
                        error_message="Failed to decode RequestMessage: " + e.args[0]
                    )
                )
                continue

            # Dispatch the request message to the target bot.
            success, err = await self.dispatch_operation(message)
            self.log(
                ("Successfully dispatched" if success else "Failed to dispatch") +
                f" RequestMessage '{message.id}' " +
                (f"to '{message.target}' " if message.target is not None else "") +
                f"({message.op})."
            )

            # If the dispatching was unsuccessful, send an error back since they won't get a normal reply.
            if not success:
                self.log("Dispatch error: " + err)
                await self.send_reply_message(
                    ReplyMessage(
                        id=message.id,
                        success=False,
                        error_message=err
                    )
                )

    async def send_reply_message(self, message: ReplyMessage) -> None:

        # Make sure we websocket is still open before sending.
        if await self.check_closed():
            await self.websocket.send(
                msgspec.json.encode(message).decode("utf-8")
            )

        else:
            self.log("Could not send reply as the socket has been closed!")

    async def handle_reply(self, reply: OperationReply) -> None:

        # Convert the reply to a dict, so it can be serialized back to JSON.
        out = {
            "id": reply.reply_id,
            "success": reply.success
        }
        if reply.success:
            out["output"] = reply.output if reply.output_exists else None

        else:
            out["error_message"] = reply.error_message if reply.error_message_exists else "Unknown error!"

        # Convert the dict into a msg struct, so it can be encoded.
        await self.send_reply_message(
            ReplyMessage(**out)
        )
