pub mod operation;
pub mod reply;

use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    thread
};
use websocket::{client::ClientBuilder, message::OwnedMessage, sync::Client};
use websocket::sync::Writer;
use websocket::websocket_base::result::WebSocketResult;
use crate::ws::operation::Operation;

// TODO: REIMPLEMENT LOGGING DIRECTLY TO THE MENU STATE.

pub struct WebsocketClient {
    writer: Option<Arc<Mutex<Writer<TcpStream>>>>,
    pub alive: Arc<Mutex<bool>>,
    queue: Arc<Mutex<Vec<Operation>>>
}
impl WebsocketClient {
    pub fn new() -> Self {
        Self {
            writer: None,
            alive: Arc::new(Mutex::new(false)),
            queue: Arc::new(Mutex::new(Vec::<Operation>::new()))
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
    pub fn run_queue(&self) -> () {

        let mut queue = self.queue.lock().unwrap();
        for operation in queue.iter_mut() {

            // Run the operation, convert the reply to binary and send back.
            let reply_binary = operation.run().to_binary();
            match self.send(OwnedMessage::Binary(reply_binary)) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        queue.clear();
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

        // Create client and connect.
        let result = ClientBuilder::new("ws://127.0.0.1:8000")
            .unwrap()
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
                            Ok(o) => {
                                operation_queue.lock().unwrap().push(o);

                                // TODO: Remove this too.
                                ws_writer
                                    .lock()
                                    .unwrap()
                                    .send_message(&OwnedMessage::Text("Successfully queued operation.".to_owned()))
                                    .unwrap_or_else(|_| eprintln!("Failed to send success reply"));
                            },
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