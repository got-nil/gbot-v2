use rglua::lua::LuaState;
use rglua::prelude::*;

#[lua_function]
fn run_queue(l: LuaState) -> Result<i32, rglua::interface::Error> {
    crate::run_queue(Some(l));
    Ok(0)
}

pub fn apply(l: LuaState) -> () {
    let lib = reg! [
        "run_queue" => run_queue
    ];
    luaL_register(l, cstr!("gtest"), lib.as_ptr());
}