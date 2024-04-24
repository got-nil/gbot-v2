use rglua::interface;
use rglua::prelude::*;
use crate::lua::fns::safe_log;

#[lua_function]
fn run_queue(l: LuaState) -> Result<i32, interface::Error> {

    let state = Some(l);

    safe_log(state, "PRE RUNNING QUEUE");
    crate::run_queue(state);
    safe_log(state, "POST RUNNING QUEUE");

    Ok(0)
}

pub fn apply(l: LuaState) -> () {

    printgm!(l, "Applying debug functions! These leave a trace in the menu state!");

    let lib = reg![
        "RunQueue" => run_queue
    ];
    luaL_register(l, cstr!("gtest"), lib.as_ptr());
}