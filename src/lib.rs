mod hooks;
mod lua;
mod ws;

#[cfg(feature="debug")]
mod debug;

use std::cell::RefCell;
use std::time::SystemTime;
use rand::distributions::Alphanumeric;
use rand::Rng;
use rglua::prelude::*;
use rglua::interface;
use crate::ws::WebsocketClient;

// Generate an identifier with the current UNIX time and a random string.
// TODO: The websocket should probably assign the bot an identifier instead of this.
fn generate_identifier() -> String {

    // Get the current unix time, or an empty string if it fails.
    let mut unix = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().to_string(),
        Err(_) => {
            String::new()
        }
    };
    let random: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(28 - unix.len())
            .map(char::from)
            .collect();

    // Push the random string to end of unix and return.
    unix.push_str(&random);
    unix
}

thread_local! {
    pub static WEBSOCKET_CLIENT: RefCell<Option<WebsocketClient>> = RefCell::new(None);
    pub static WEBSOCKET_IDENTIFIER: String = generate_identifier();
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