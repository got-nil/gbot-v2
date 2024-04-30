use std::ffi::{c_int, CString};
use rglua::lua::{lua_pushstring, lua_toboolean, lua_tonumber, lua_tostring, lua_type, LuaState, TBOOLEAN, TNIL, TNUMBER, TSTRING, TTABLE};
use rglua::{printgm, rstr};
use rglua::prelude::dump_stack;
use crate::lua::table::LuaTable;
use crate::lua::types::LuaReturnValue;

pub fn log(state: LuaState, string: &str) -> () {
    printgm!(state, "{}", string);
}

pub fn safe_log(state: Option<LuaState>, string: &str) -> () {
    if let Some(l) = state {
        log(l, string);
    }
}

pub fn log_dump_stack(state: LuaState) -> () {
    match dump_stack(state) {
        Ok(string) => log(state, string.as_str()),
        Err(_) => log(state, "Could not log state!")
    }
}

pub fn lua_push_string(state: LuaState, string: String) -> Result<(), &'static str> {

    // Convert String into CString.
    let cstr = match CString::new(string) {
        Ok(cstr) => cstr,
        Err(_) => {
            return Err("Could not convert String into a CString!");
        }
    };

    // Actually push the CString.
    lua_pushstring(state, cstr.as_ptr());
    Ok(())
}

pub fn lua_get_return_value(state: LuaState, idx: c_int) -> Option<LuaReturnValue> {

    // Get the value (as return value type) from stack.
    match lua_type(state, idx) {
        TBOOLEAN => Some(LuaReturnValue::Bool(lua_toboolean(state, idx) == 1)),
        TNUMBER => Some(LuaReturnValue::Number(lua_tonumber(state, idx))),
        TSTRING => Some(LuaReturnValue::String(rstr!(lua_tostring(state, idx)).to_string())),
        TNIL => Some(LuaReturnValue::Nil),
        TTABLE => Some(LuaReturnValue::Table(Box::new(LuaTable::read_table(state, idx)))),
        _ => None
    }
}