from enum import IntEnum
from typing import TYPE_CHECKING, Optional, List, Any, Tuple
from dataclasses import dataclass

from binary.encoder import Encoder
from binary.decoder import Decoder

if TYPE_CHECKING:
    from bot import Bot


class OperationType(IntEnum):

    # Bot enums.
    ConnectToServer = 1
    DisconnectFromServer = 2
    ExecuteLua = 3
    ExitProcess = 4

    # Server enums.
    ListBots = 5

    # IsBotOperation, RequiresPayload
    def meta(self) -> Tuple[bool, bool]:
        return 1 <= self.value <= 4, self.value in [1, 2, 4]

    # Called when an operation type is sent, with the bot that did it.
    # This is only for operations that send data to the bot and should also
    # have some immediate effect here too (such as ExitProcess closing socket).
    # Returns if the Operation should be cached as pending.
    async def sent(self, bot: "Bot") -> bool:
        match self.value:
            case 4:
                await bot.close()
                return False

            case _:
                return True


@dataclass
class Operation:
    reply_id: str
    type: OperationType
    payloads: List[str]

    def to_bytes(self) -> bytes:

        encoder = Encoder()
        encoder.write_string(self.reply_id)  # REPLY ID
        encoder.write_u8(self.type.value)  # OPERATION TYPE
        encoder.write_u8(len(self.payloads))  # PAYLOAD COUNT

        # Write operation payloads.
        for payload in self.payloads:
            encoder.write_string(payload)

        # Return bytearray as one 'bytes' object.
        return bytes(encoder.buffer)


@dataclass
class OperationReply:
    reply_id: str
    success: bool

    # Error handling.
    error_message_exists: Optional[bool]
    error_message: Optional[str]

    # Protected call output.
    output_exists: Optional[bool]
    output: Optional[List[Any]]

    @classmethod
    def from_bytes(cls, buffer: bytes) -> Optional["OperationReply"]:

        decoder = Decoder(buffer)
        reply = {
            "reply_id": decoder.read_string(),
            "success": decoder.read_bool(),

            # Default values.
            "error_message_exists": None,
            "error_message": None,
            "output_exists": None,
            "output": None
        }

        # Read error message.
        if not reply["success"]:
            reply["error_message_exists"] = decoder.read_bool()

            if reply["error_message_exists"]:
                reply["error_message"] = decoder.read_string()

        else:
            reply["output_exists"] = decoder.read_bool()

            # Read outputted lua types if there are any.
            if reply["output_exists"]:

                output = []
                for i in range(decoder.read_u32()):
                    output.append(
                        decoder.read_lua_type()
                    )

                reply["output"] = output

        return cls(**reply)


@dataclass
class PendingOperation:
    client_id: str
    operation: Operation
