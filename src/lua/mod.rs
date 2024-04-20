pub mod types;

use std::ffi::CStr;
use rglua::prelude::*;
use crate::hooks::LUAL_LOADBUFFERX_H;
use crate::lua::types::{LuaPayload, LuaResult, LuaReturnValue, Realm};

unsafe fn stack_get_error(state: LuaState) -> String {
    let err = lua_tostring(state, -1);
    lua_pop(state, 1);

    let err = CStr::from_ptr(err);
    err.to_string_lossy().to_string()
}

pub fn get_state(realm: Realm) -> Result<LuaState, rglua::interface::Error> {
    let shared = iface!(LuaShared)?;
    let interface = unsafe { shared.GetLuaInterface(realm.into()).as_mut() }
        .ok_or(rglua::interface::Error::AsMut)?;

    Ok(interface.base.cast())
}

pub fn run(payload: LuaPayload) -> LuaResult {

    // Get state requested by the payload.
    match get_state(payload.realm) {
        Ok(state) => {

            // Compile code.
            let str = payload.code.as_str();
            unsafe {
                if LUAL_LOADBUFFERX_H.call(
                    state,
                    str.as_ptr().cast(),
                    str.len(),
                    cstr!("@RunString"),
                    cstr!("bt")
                ) != OK {
                    return LuaResult {
                        success: false,
                        error_message: Some(stack_get_error(state)),
                        output: None
                    };
                }
            }

            // Execute compiled code in a protected call.
            if lua_pcall(state, 0, MULTRET, 0) != OK {
                unsafe {
                    return LuaResult {
                        success: false,
                        error_message: Some(stack_get_error(state)),
                        output: None,
                    }
                }
            };

            // Get stack size and create output vec.
            let top = lua_gettop(state);
            let mut out = Vec::<LuaReturnValue>::new();

            // Go through the stack backwards.
            for _ in 1..=top {

                // Get the value (as return value type) from stack & pop.
                let ret = match lua_type(state, -1) {
                    TBOOLEAN => Some(LuaReturnValue::Bool(lua_toboolean(state, -1) == 1)),
                    TNUMBER => Some(LuaReturnValue::Number(lua_tonumber(state, -1))),
                    TSTRING => Some(LuaReturnValue::String(rstr!(lua_tostring(state, -1)).to_string())),
                    TNIL => Some(LuaReturnValue::Nil),
                    _ => None
                };
                lua_pop(state, 1);

                // Make sure there's actually a valid return value.
                if let Some(value) = ret {
                    out.push(value);
                }
            }

            // Reverse the output since we read it backwards.
            out.reverse();

            LuaResult {
                success: true,
                error_message: None,
                output: Some(out)
            }
        },
        Err(_) => {
            LuaResult {
                success: false,
                error_message: Some(String::from("Could not get realm!")),
                output: None
            }
        }
    }
}