import struct
from typing import Union
from binary.debug_logger import DebugLogger


class Decoder(DebugLogger):

    buffer: bytes
    position: int

    def __init__(self, buffer: bytes):
        self.buffer = buffer
        self.position = 0

    def read_bool(self) -> bool:
        self.debug("Reading boolean.")
        out = struct.unpack_from("<B", self.buffer, self.position)
        self.position += 1
        return bool(out[0])

    def read_float64(self) -> float:
        self.debug("Reading float64.")
        out = struct.unpack_from("<d", self.buffer, self.position)
        self.position += 8
        return float(out[0])

    def read_string(self) -> str:
        self.debug("Reading string.")
        length = self.read_u32()
        string = self.buffer[self.position:self.position+length].decode("utf-8")
        self.position += length
        return string

    def read_u8(self) -> int:
        self.debug("Reading u8.")
        out = struct.unpack_from("<B", self.buffer, self.position)
        self.position += 1
        return out[0]

    def read_u32(self) -> int:
        self.debug("Reading u32.")
        out = struct.unpack_from("<I", self.buffer, self.position)
        self.position += 4
        return out[0]

    def read_lua_type(self) -> Union[str, bool, float, None]:

        # Type readers.
        self.debug("Reading lua type.")
        readers = {
            1: self.read_bool,
            2: self.read_float64,
            3: self.read_string,
            4: lambda: None
        }

        lua_type = self.read_u8()
        if lua_type not in readers:
            raise Exception("Could not find valid lua type reader!")

        return readers[lua_type]()

