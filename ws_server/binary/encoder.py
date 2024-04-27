import struct


class Encoder:

    buffer: bytearray

    def __init__(self):
        self.buffer = bytearray()

    def write_bool(self, value: bool) -> None:
        self.buffer.extend(struct.pack("<B", int(value)))

    def write_float64(self, value: float) -> None:
        self.buffer.extend(struct.pack("<d", value))

    def write_string(self, value: str) -> None:
        encoded = value.encode("utf-8")
        self.write_u32(len(encoded))
        self.buffer.extend(encoded)

    def write_u8(self, value: int) -> None:
        self.buffer.extend(struct.pack("<B", value))

    def write_u32(self, value: int) -> None:
        self.buffer.extend(struct.pack("<I", value))