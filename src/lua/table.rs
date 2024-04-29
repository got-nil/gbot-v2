use std::fmt::{Display, Formatter};
use rglua::lua::LuaState;
use rglua::util::dump_stack;
use crate::lua::fns::safe_log;
use crate::lua::types::LuaReturnValue;

pub struct LuaTable {
    key: LuaReturnValue,
    value: LuaReturnValue
}
impl LuaTable {

    pub fn read_table(state: LuaState) -> Self {

        // TODO: Actually read the table from stack.
        unimplemented!();

        LuaTable {
            key: LuaReturnValue::Nil,
            value: LuaReturnValue::Nil,
        }
    }
}
impl Display for LuaTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}: {}}}", self.key.to_string(), self.value.to_string())
    }
}