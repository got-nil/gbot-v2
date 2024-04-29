pub mod operation;
pub mod reply;

use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    thread
};
use std::time::{Duration, Instant};
use rglua::lua::LuaState;
use websocket::{client::ClientBuilder, message::OwnedMessage, sync::Client};
use websocket::header::Headers;
use websocket::sync::Writer;
use websocket::websocket_base::result::WebSocketResult;
use crate::lua::fns::safe_log;
use crate::WEBSOCKET_IDENTIFIER;
use crate::ws::operation::Operation;

// How long to delay before allowing reconnection attempts after a failure.
static RECONNECT_ATTEMPT_DURATION: Duration = Duration::from_secs(10);

pub struct WebsocketClient {
    pub alive: Arc<Mutex<bool>>,
    pub queue: Arc<Mutex<Vec<Operation>>>,
    writer: Option<Arc<Mutex<Writer<TcpStream>>>>,
    last_connection_attempt: Option<Instant>
}
impl WebsocketClient {
    pub fn new() -> Self {
        Self {
            alive: Arc::new(Mutex::new(false)),
            queue: Arc::new(Mutex::new(Vec::<Operation>::new())),
            writer: None,
            last_connection_attempt: None
        }
    }

    // Actually connect to the websocket and start the ping + receive loops.
    pub fn start(&mut self) -> Result<(), &'static str> {

        // Don't start if we're already alive.
        if self.is_alive() {
            return Err("The websocket connection is already alive!");
        }

        // Get websocket client.
        // TODO: Requeue for failures?
        let client = match self.connect() {
            Ok(client) => client,
            Err(e) => {
                return Err(e);
            }
        };

        self.writer = Some(
            self.create_thread(client)
        );
        Ok(())
    }

    // Check if the websocket is alive / connected.
    pub fn is_alive(&self) -> bool {
        *self.alive.lock().unwrap()
    }

    // Send a message to an opened writer.
    pub fn send(&self, message: OwnedMessage) -> Result<WebSocketResult<()>, &'static str> {
        if !self.is_alive() {
            return Err("Connection is not alive.");
        }
        match &self.writer {
            Some(writer) => {
                Ok(writer
                    .lock()
                    .unwrap()
                    .send_message(&message))
            },
            None => {
                Err("Somehow the connection is alive, but there is no writer?")
            }
        }
    }

    // Run every queued operation with a lua state. (then clear).
    pub fn run_queue(&self, state: Option<LuaState>) -> bool {

        // If the connection isn't alive, don't bother running operations since
        // we can't actually send back any replies.
        if !self.is_alive() {
            safe_log(state, "Could not run queue as the connection is dead!");
            return false;
        }

        // Lock the queue.
        let mut queue = match self.queue.lock() {
            Ok(queue) => queue,
            Err(_) => {
                safe_log(state, "Could not acquire a lock for the operation queue!");
                return true;
            }
        };

        // Make sure the queue isn't empty.
        if queue.len() == 0 {
            safe_log(state, "The queue is empty!");
            return true;
        }

        // Pop the first operation in queue.
        let mut operation = queue.remove(0);
        if state.is_some() {
            safe_log(state, format!("{operation:?}").as_str());
        }

        // Get operation reply data.
        let reply = operation.run();
        if state.is_some() {
            safe_log(state, format!("{reply:?}").as_str());
        }
        match self.send(OwnedMessage::Binary(reply.to_binary())) {
            Ok(_) => {}
            Err(_) => {
                safe_log(state, "Failed to send operation reply!");
            }
        }
        return true;
    }

    // Attempt to reconnect to the websocket. Will only allow reconnection attempt every
    // RECONNECT_ATTEMPT_DURATION, therefore it can be called any time run_queue fails.
    pub fn attempt_reconnection(&mut self) -> () {

        let now = Instant::now();

        // Make sure the last attempt wasn't too recent.
        if let Some(last_attempt) = self.last_connection_attempt {
            let elapsed = now.duration_since(last_attempt);
            if elapsed < RECONNECT_ATTEMPT_DURATION {
                return;
            }
        }
        self.last_connection_attempt = Some(now);

        // Actually attempt the connection start.
        // TODO: Add proper logging.
        let _ = self.start();
    }

    // "Reset" the client, so it can get ready to handle a new connection.
    fn reset(&mut self) -> () {

        // Reset writer.
        self.writer = None;

        // Mark as not alive and clear queue.
        *self.alive.lock().unwrap() = false;
        self.queue.lock().unwrap().clear();
    }

    // Connect to a websocket.
    fn connect(&mut self) -> Result<Client<TcpStream>, &'static str> {

        // Reset the client before connecting.
        self.reset();

        // Create headers set with bot identifier.
        let mut headers = Headers::new();
        headers.set_raw("X-Id", vec![WEBSOCKET_IDENTIFIER.with(|id| id.clone()).as_bytes().to_vec()]);

        // Create client and connect.
        let result = ClientBuilder::new("ws://127.0.0.1:8000/ws/bot")
            .unwrap()
            .custom_headers(&headers)
            .connect_insecure();

        match result {
            Ok(client) => {
                *self.alive.lock().unwrap() = true;
                Ok(client)
            },
            Err(_) => {
                Err("Could not connect to websocket!")
            }
        }
    }

    // Actually create the read thread and return stream writer.
    fn create_thread(&mut self, client: Client<TcpStream>) -> Arc<Mutex<Writer<TcpStream>>> {

        // Get stream reader and writer.
        let (mut reader, writer) = client.split().unwrap();
        let ws_writer = Arc::new(Mutex::new(writer));
        let return_writer = Arc::clone(&ws_writer);

        // Websocket receiver thread.
        let websocket_alive = Arc::clone(&self.alive);
        let operation_queue = Arc::clone(&self.queue);
        thread::spawn(move || {

            // Block while connection is open, receiving all messages.
            for message in reader.incoming_messages() {
                let message = match message {
                    Ok(message) => message,
                    Err(e) => {
                        eprintln!("Error receiving message: {:?}", e);
                        break;
                    }
                };

                match message {
                    OwnedMessage::Binary(bin) => {

                        match Operation::from_binary(bin) {
                            Ok(o) => operation_queue.lock().unwrap().push(o),
                            Err(e) => {

                                // TODO: Remove this? We just shouldn't be sending invalid operations anyway.
                                ws_writer
                                    .lock()
                                    .unwrap()
                                    .send_message(&OwnedMessage::Text(e))
                                    .unwrap_or_else(|_| eprintln!("Failed to send error reply"));
                            }
                        }
                    },
                    OwnedMessage::Ping(data) => {
                        ws_writer
                            .lock()
                            .unwrap()
                            .send_message(&OwnedMessage::Pong(data))
                            .unwrap_or_else(|e| eprintln!("Failed to send pong: {:?}", e))
                    }
                    OwnedMessage::Close(_) => {

                        // TODO: Exit process since the master server is shutting down?
                        println!("Websocket gracefully closing!");
                    },
                    _ => {}
                }
            }

            // If the reader has stopped then the connection is dead.
            *websocket_alive.lock().unwrap() = false;
            println!("Websocket end of receiver thread!");
        });

        return_writer
    }
}