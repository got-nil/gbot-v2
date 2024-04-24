mod hooks;
mod lua;
mod ws;
mod debug;

use std::cell::RefCell;
use rglua::prelude::*;
use rglua::interface;
use crate::ws::WebsocketClient;

thread_local! {
    pub static WEBSOCKET_CLIENT: RefCell<Option<WebsocketClient>> = RefCell::new(None);
}

// Run operation queue.
// This is called basically every frame, coming from the paint traverse.
fn run_queue(state: Option<LuaState>) -> () {
    WEBSOCKET_CLIENT.with(|cell| {
        if let Some(websocket) = cell.borrow_mut().as_mut() {

            // Run the websocket queue, and attempt reconnection if it fails (socket is dead).
            if !websocket.run_queue(state) {
                websocket.attempt_reconnection();
            }
        }
    });
}

#[gmod_open]
fn open(l: LuaState) -> Result<i32, interface::Error> {

    #[cfg(feature = "debug")]
    debug::apply(l);

    // Attach detour hooks.
    match hooks::attach() {
        Ok(_) => printgm!(l, "Successfully attached hooks!"),
        Err(why) => printgm!(l, "Failed to attach hooks: {why:#?}")
    }

    // Create websocket & attempt connection.
    let mut websocket = WebsocketClient::new();
    let start_result = websocket.start();
    WEBSOCKET_CLIENT.with(|cell| {
        let mut inner = cell.borrow_mut();
        *inner = Some(websocket);
    });

    // Log initial websocket connection status.
    match start_result {
        Ok(_) => printgm!(l, "Successfully connected to websocket!"),
        Err(e) => printgm!(l, "Failed to connect to websocket: {e}")
    }

    Ok(0)
}

#[gmod_close]
fn close(l: LuaState) -> i32 {

    // TODO: Actually close the websocket connection here.

    match hooks::detach() {
        Ok(_) => {
            printgm!(l, "Successfully detached hooks!");
            0
        },
        Err(_) => {
            printgm!(l, "Failed to detach hooks!");
            1
        }
    }
}