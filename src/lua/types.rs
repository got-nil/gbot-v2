use std::fmt;
use std::fmt::{Display, Formatter};
use crate::lua::table::LuaTable;
use crate::ws::binary::{BinaryBuffer, BinaryWriter};

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
    Nil,
    Table(Box<LuaTable>),
}
impl LuaReturnValue {
    fn to_uint(&self) -> u8 {
        match self {
            LuaReturnValue::Bool(_) => 1u8,
            LuaReturnValue::Number(_) => 2u8,
            LuaReturnValue::String(_) => 3u8,
            LuaReturnValue::Nil => 4u8,
            LuaReturnValue::Table(_) => 5u8
        }
    }
}
impl Display for LuaReturnValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let v = match self {
            LuaReturnValue::Bool(b) => if *b { "true" } else { "false" }.to_owned(),
            LuaReturnValue::Number(f) => f.to_string(),
            LuaReturnValue::String(s) => format!("\"{}\"", s.clone()),
            LuaReturnValue::Nil => "nil".to_owned(),
            LuaReturnValue::Table(t) => t.to_string()
        };
        write!(f, "{}", v)
    }
}
impl BinaryWriter for LuaReturnValue {
    fn write_binary(&self, buffer: &mut BinaryBuffer) -> Result<(), &str> {

        // Write the value type.
        buffer.write_u8(self.to_uint());

        // Actually write the value.
        match self {
            LuaReturnValue::Bool(b) => buffer.write_bool(*b),
            LuaReturnValue::Number(f) => buffer.write_f64(*f),
            LuaReturnValue::String(s) => buffer.write_str(s.as_str()),
            LuaReturnValue::Nil => {},
            LuaReturnValue::Table(t) => {
                if let Err(e) =  t.write_binary(buffer) {
                    return Err(e);
                }
            }
        }
        Ok(())
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