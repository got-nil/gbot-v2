mod hooks;
mod lua;
mod ws;
mod debug;

use std::cell::OnceCell;
use std::ffi::CString;
use rglua::prelude::*;
use rglua::interface;
use crate::ws::WebsocketClient;

thread_local! {
    pub static WEBSOCKET_CLIENT: OnceCell<Option<WebsocketClient>> = OnceCell::new();
}

// Run operation queue.
fn run_queue() -> () {

    // TODO: Reconnect websocket if its not alive.

    WEBSOCKET_CLIENT.with(|cell| {
        if let Some(websocket) = cell.get() {
            match websocket {
                None => {}
                Some(ws) => {
                    ws.run_queue()
                }
            }
        }
    });
}

#[gmod_open]
fn open(l: LuaState) -> Result<i32, interface::Error> {

    // TODO: REMOVE DEBUG ATTACHMENTS.
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
        match cell.set(Some(websocket)) {
            Ok(_) => {}
            Err(_) => {

                // Usually we shouldn't panic, but if we can't store the websocket client then there is
                // literally no way for operations to be passed to the game, so we might as well exit now.
                // TODO: Return an interface Error instead!
                panic!("Could not store WebsocketClient in OnceCell!");
            }
        }
    });

    // Log initial websocket connection status.
    match start_result {
        Ok(_) => printgm!(l, "Successfully connected to websocket!"),
        Err(_) => printgm!(l, "Failed to connect to websocket!")
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