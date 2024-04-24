use std::fmt;
use std::fmt::{Display, Formatter};

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Realm {
    Client = 0,
    Server = 1,
    Menu = 2
}
impl From<Realm> for u8 {
    fn from(realm: Realm) -> u8 {
        match realm {
            Realm::Client => 0,
            Realm::Server => 1,
            Realm::Menu => 2
        }
    }
}

pub enum LuaReturnValue {
    Bool(bool),
    Number(f64),
    String(String),
    Nil
}
impl LuaReturnValue {
    fn to_uint(&self) -> u8 {
        match self {
            LuaReturnValue::Bool(_) => 1u8,
            LuaReturnValue::Number(_) => 2u8,
            LuaReturnValue::String(_) => 3u8,
            LuaReturnValue::Nil => 4u8
        }
    }
    pub fn write_binary(&self, encoded: &mut Vec<u8>) -> () {

        // Write the value type.
        encoded.extend_from_slice(
            &self.to_uint().to_le_bytes()
        );

        // Actually write the value.
        match self {
            LuaReturnValue::Bool(b) => encoded.push(if *b { 0x01 } else { 0x00 }),
            LuaReturnValue::Number(f) => encoded.extend_from_slice(&f.to_le_bytes()),
            LuaReturnValue::String(s) => {

                // Convert the String to utf8 bytes.
                let utf8_bytes = s.as_bytes();

                // Write the string size and bytes.
                encoded.extend_from_slice(&(utf8_bytes.len() as u32).to_le_bytes());
                encoded.extend_from_slice(utf8_bytes);
            }
            LuaReturnValue::Nil => {}
        }
    }
}
impl Display for LuaReturnValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            LuaReturnValue::Bool(b) => if *b { "true" } else { "false" }.to_owned(),
            LuaReturnValue::Number(f) => f.to_string(),
            LuaReturnValue::String(s) => s.clone(),
            LuaReturnValue::Nil => "nil".to_owned()
        };
        write!(f, "{}", v)
    }
}
impl fmt::Debug for LuaReturnValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("LuaReturnValue")
            .field("type", &self.to_uint())
            .field("value", &self.to_string())
            .finish()
    }
}

pub struct LuaPayload {
    pub realm: Realm,
    pub code: String
}

pub struct LuaResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub output: Option<Vec<LuaReturnValue>>
}