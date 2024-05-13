use std::fmt::{Display, Formatter};
use rglua::lua::{lua_next, lua_pop, lua_pushnil, LuaState};
use crate::lua::fns::lua_get_return_value;
use crate::lua::types::LuaReturnValue;
use crate::ws::binary::{BinaryBuffer, BinaryWriter};

/*
    TODO: Implement max table depth checking. Also maybe a max table size
          just encase to make sure the while loop doesn't stall. The stack
          still must be cleared afterwards since we're reading sequentially.
 */

struct LuaTableItem {
    key: LuaReturnValue,
    value: LuaReturnValue
}
impl BinaryWriter for LuaTableItem {
    fn write_binary(&self, buffer: &mut BinaryBuffer) -> Result<(), &str> {
        if self.key.write_binary(buffer).is_err() ||
           self.value.write_binary(buffer).is_err()
        {
            return Err("Could not write table KeyValue item.");
        }
        Ok(())
    }
}
impl Display for LuaTableItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.key.to_string(), self.value.to_string())
    }
}

pub struct LuaTable {
    items: Vec<LuaTableItem>
}
impl LuaTable {

    pub fn read_table(state: LuaState) -> Self {

        // Required for table traversal.
        lua_pushnil(state);

        let mut items = Vec::<LuaTableItem>::new();
        while lua_next(state, -2) != 0 {

            // Read table key. It must be valid.
            if let Some(k) = lua_get_return_value(state, -2) {

                // Read value. If it's invalid, default to nil.
                let v = lua_get_return_value(state, -1).unwrap_or_else(|| LuaReturnValue::Nil);
                items.push(
                    LuaTableItem {
                        key: k,
                        value: v
                    }
                );
            }
            lua_pop(state, 1);
        }

        // The original table is still on the stack here, but it should
        // be left to be popped by the caller to remain consistent.

        LuaTable {
            items
        }
    }
}
impl BinaryWriter for LuaTable {
    fn write_binary(&self, buffer: &mut BinaryBuffer) -> Result<(), &str> {
        buffer.write_u32(self.items.len() as u32);
        for item in &self.items {
            if let Err(e) = item.write_binary(buffer) {
                return Err(e);
            }
        }
        Ok(())
    }
}
impl Display for LuaTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", self.items
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join(", ")
        )
    }
}