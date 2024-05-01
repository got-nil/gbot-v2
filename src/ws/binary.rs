
pub trait BinaryWriter {
    fn write_binary(&self, buffer: &mut BinaryBuffer) -> Result<(), &str>;
}

pub struct BinaryBuffer {
    buffer: Vec<u8>
}
impl BinaryBuffer {
    pub fn new() -> Self {
        BinaryBuffer {
            buffer: Vec::<u8>::new()
        }
    }

    pub fn write_str(&mut self, v: &str) -> () {

        let utf8_bytes = v.as_bytes();
        let length = utf8_bytes.len() as u32;

        self.buffer.extend_from_slice(&length.to_le_bytes()); // String size
        self.buffer.extend_from_slice(utf8_bytes); // String value
    }
    pub fn write_bool(&mut self, v: bool) -> () {
        self.buffer.push(if v { 0x01 } else { 0x00 });
    }
    pub fn write_u32(&mut self, v: u32) -> () {
        self.buffer.extend_from_slice(&v.to_le_bytes());
    }
    pub fn write_u8(&mut self, v: u8) -> () {
        self.buffer.extend_from_slice(&v.to_le_bytes());
    }
    pub fn write_f64(&mut self, v: f64) -> () {
        self.buffer.extend_from_slice(&v.to_le_bytes());
    }
}
impl Into<Vec<u8>> for BinaryBuffer {
    fn into(self) -> Vec<u8> {
        self.buffer
    }
}