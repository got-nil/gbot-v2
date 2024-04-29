use std::panic::catch_unwind;
use rglua::interface;
use rglua::prelude::*;
use crate::lua::fns::safe_log;
use crate::WEBSOCKET_CLIENT;

#[lua_function]
fn run_queue(l: LuaState) -> Result<i32, interface::Error> {

    let state = Some(l);

    let result = catch_unwind(|| {
        safe_log(state, "PRE RUNNING QUEUE");
        crate::run_queue(state);
        safe_log(state, "POST RUNNING QUEUE");
    });

    safe_log(state, if result.is_ok() { "OK" } else { "ERR" });
    Ok(0)
}

#[lua_function]
fn check_queue(l: LuaState) -> Result<i32, interface::Error> {

    WEBSOCKET_CLIENT.with(|cell| {
        if let Some(websocket) = cell.borrow().as_ref() {

            printgm!(l, "Got websocket from cell!");
            for op in websocket.queue.lock().unwrap().iter() {
                printgm!(l, "{op:#?}");
            }
        } else {
            printgm!(l, "Could not get websocket from cell!");
        }
    });

    Ok(0)
}

pub fn apply(l: LuaState) -> () {

    printgm!(l, "Applying debug functions! These leave a trace in the menu state!");

    let lib = reg![
        "RunQueue" => run_queue,
        "CheckQueue" => check_queue
    ];
    luaL_register(l, cstr!("gtest"), lib.as_ptr());
}