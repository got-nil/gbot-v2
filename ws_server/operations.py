from enum import IntEnum
from typing import Optional, List, Any
from dataclasses import dataclass

from binary.encoder import Encoder
from binary.decoder import Decoder


class OperationType(IntEnum):
    Error = 0
    ConnectToServer = 1
    DisconnectFromServer = 2
    ExecuteLua = 3
    ExitProcess = 4

    @classmethod
    def from_str(cls, string: str) -> Optional["OperationType"]:
        lookup = {
            "Error": cls.Error,
            "ConnectToServer": cls.ConnectToServer,
            "DisconnectFromServer": cls.DisconnectFromServer,
            "ExecuteLua": cls.ExecuteLua,
            "ExitProcess": cls.ExitProcess
        }
        return lookup.get(string)


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
            "id": decoder.read_string(),
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
