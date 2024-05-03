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

    def read_lua_type(self) -> Union[str, bool, float, dict, list, None]:

        # Type readers.
        readers = {
            1: ["bool", self.read_bool],
            2: ["f64", self.read_float64],
            3: ["string", self.read_string],
            4: ["nil", lambda: None],
            5: ["table", self.read_table]
        }

        lua_type = self.read_u8()
        if lua_type not in readers:
            raise Exception("Could not find valid lua type reader!")

        type_name, type_fn = readers[lua_type]
        self.debug(f"Reading lua type {lua_type} ({type_name})")

        return type_fn()

    def read_table(self) -> Union[dict, list, None]:

        tbl, is_sequential, last_k = {}, True, None

        # LuaTable
        items_size = self.read_u32()
        for i in range(items_size):

            # LuaTableKeyValue
            k = self.read_lua_type()
            v = self.read_lua_type()

            if k is None or v is None:
                return None

            # If the key is a float, convert it to an int if it's actually an integer.
            is_int = False
            if type(k) is float and k.is_integer():
                is_int, k = True, int(k)

            # Check if the table is still sequential (integer keys in order)
            if is_sequential:
                if last_k is None: is_sequential = k == 1
                else: is_sequential = (is_int or type(k) is int) and k == last_k + 1
                last_k = k

            tbl[k] = v

        # If the table is sequential, convert it to a list.
        if is_sequential:
            return [tbl[i + 1] for i in range(items_size)]

        return tbl
