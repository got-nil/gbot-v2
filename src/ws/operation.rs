use std::fmt;
use std::fmt::Formatter;
use std::process::exit;
use rglua::cstr;
use rglua::lua::{lua_call, lua_getglobal, lua_pushstring, LuaState};
use crate::lua;
use crate::lua::fns::lua_push_string;
use crate::lua::types::{LuaPayload, LuaReturnValue, Realm};
use crate::ws::reply::OperationReply;

/*

    Operation message structure:
        reply_id - ( size: u32, utf: [u8]*size )
        op_type - u8
        payload_count: u8

        (REPEATED FOR payload_count)
            payload_value - ( size: u32, utf: [u8]*size )

 */

#[derive(Debug)]
pub enum OperationType {
    ConnectToServer,
    DisconnectFromServer,
    ExecuteLua,
    ExitProcess
}
impl OperationType {
    fn from_u8(u8: u8) -> Option<OperationType> {
        match u8 {
            1u8 => Some(OperationType::ConnectToServer),
            2u8 => Some(OperationType::DisconnectFromServer),
            3u8 => Some(OperationType::ExecuteLua),
            4u8 => Some(OperationType::ExitProcess),
            _ => None
        }
    }
}

pub struct Operation {
    pub id: String,
    op_type: OperationType,
    payloads: Vec<String>
}
impl Operation {

    pub fn new(id: String, op_type: OperationType, payloads: Vec<String>) -> Self {
        Operation {
            id,
            op_type,
            payloads
        }
    }

    // Decode the packed binary operation data.
    pub fn from_binary(bin: Vec<u8>) -> Result<Self, String> {

        let bin_size = bin.len();
        let mut pos = 0;

        // Read a string and advance position.
        fn read_str(bin: &[u8], bin_size: usize, pos: &mut usize) -> Result<String, String> {

            // Check that there is enough room for the length.
            if *pos + 4 > bin_size {
                return Err("Invalid string size for length!".to_owned());
            }

            // Read length (4 bytes).
            let length_bytes = &bin[*pos..*pos + 4];
            let length = u32::from_le_bytes([
                length_bytes[0], length_bytes[1],
                length_bytes[2], length_bytes[3]
            ]);
            *pos += 4;

            // Check that there is enough room for the value.
            let end_pos = *pos + length as usize;
            if end_pos > bin_size {
                return Err("Invalid size for value.".to_owned());
            }

            // Read value.
            let value_bytes = &bin[*pos..end_pos];
            let out = match String::from_utf8(value_bytes.to_vec()) {
                Ok(str) => Ok(str),
                Err(_) => {
                    Err("Could not decode value utf8 bytes.".to_owned())
                }
            };
            *pos = end_pos;
            out
        }
        fn read_u8(bin: &[u8], bin_size: usize, pos: &mut usize) -> Result<u8, String> {

            // Check that there is an additional byte to read.
            if *pos + 1 > bin_size {
                return Err("Invalid size for u8!".to_owned())
            }

            // Read u8 (I have no idea why it has to be done like this).
            let byte = &bin[*pos];
            let result = u8::from_le_bytes([
                *byte
            ]);
            *pos += 1;
            Ok(result)
        }

        // 1. Read reply ID.
        let id = match read_str(&bin, bin_size, &mut pos) {
            Ok(reply_id) => reply_id,
            Err(_) => {
                return Err("Could not read reply ID!".to_owned())
            }
        };

        // 2. Read operation type.
        let op_type = match read_u8(&bin, bin_size, &mut pos) {
            Ok(v) => match OperationType::from_u8(v) {
                None => {
                    return Err("Invalid operation type!".to_owned())
                },
                Some(op_type) => op_type
            }
            Err(_) => {
                return Err("Could not read operation type!".to_owned())
            }
        };

        // 3. Read payload count.
        let payload_count = match read_u8(&bin, bin_size, &mut pos) {
            Ok(payload_count) => payload_count,
            Err(_) => {
                return Err("Could not read payload count!".to_owned())
            }
        };
        if payload_count > 2 {
            return Err("There cannot be more than two payload values!".to_owned())
        }

        // 4. If there are payloads, read them.
        let mut payloads = Vec::<String>::with_capacity(payload_count as usize);
        if payload_count > 0 {
            for i in 0..payload_count {

                // Read the payload value.
                match read_str(&bin, bin_size, &mut pos) {
                    Ok(v) => payloads.push(v),
                    Err(_) => {
                        return Err(format!("Could not read payload value #{i}").to_string())
                    }
                };
            }
        }

        // Finally, return the constructed Operation.
        Ok(
            Operation::new(id, op_type, payloads)
        )
    }

    // Actually run the operation!
    pub fn run(&mut self) -> OperationReply {

        // Short constructor.
        let replier = |success: bool, output: Option<Vec<LuaReturnValue>>, error_message: Option<String>| -> OperationReply {
            OperationReply::new(
                self.id.clone(),
                success,
                output,
                error_message
            )
        };

        return match self.op_type {

            // Join a game server.
            OperationType::ConnectToServer => {

                // There must be one payload (the server IP).
                if self.payloads.len() != 1 {
                    return replier(false, None, Some("Missing required payload! (Server IP)".to_owned()));
                }

                // Get server IP from payload.
                let server_ip = match self.payloads.pop() {
                    Some(server_ip) => server_ip,
                    None => {
                        return replier(false, None, Some("Could not get valid server IP payload!".to_owned()));
                    }
                };

                // Get the menu state.
                let l = match lua::get_state(Realm::Menu) {
                    Ok(l) => l,
                    Err(_) => {
                        return replier(false, None, Some("Could not get menu state to JoinServer!".to_owned()))
                    }
                };

                // Actually call the join server function and reply.
                lua_getglobal(l, cstr!("JoinServer"));
                if let Err(_) = lua_push_string(l, server_ip) {
                    return replier(false, None, Some("Could not push server IP payload to stack!".to_owned()));
                }
                lua_call(l, 1, 0);
                replier(true, None, None)
            },

            // Disconnect from the game server.
            OperationType::DisconnectFromServer => {
                unimplemented!()
            },

            // Execute a lua payload and return its outputs.
            OperationType::ExecuteLua => {

                // There must be two payloads here. The realm identifier and the code.
                if self.payloads.len() != 2 {
                    return replier(false, None, Some("Missing required two payloads! (Realm, Code)".to_owned()));
                }

                // Read payload values.
                let code = match self.payloads.pop() {
                    None => {
                        return replier(false, None, Some("Could not pop payload!".to_owned()));
                    },
                    Some(code) => code
                };
                let realm = match self.payloads.pop() {
                    None => {
                        return replier(false, None, Some("Could not pop realm!".to_owned()));
                    }
                    Some(realm) => match realm.as_str() {
                        "c" => Realm::Client,
                        "m" => Realm::Menu,
                        _ => Realm::Menu
                    }
                };

                // Run, and convert the LuaResult to an OperationReply.
                let payload = LuaPayload {
                    realm,
                    code
                };
                OperationReply::from((
                    self.id.clone(), lua::run(payload)
                ))
            },

            // Exit the game process.
            OperationType::ExitProcess => {
                replier(true, None, None);
                exit(1);
            }
        }
    }
}
impl fmt::Debug for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Operation")
            .field("id", &self.id)
            .field("type", &self.op_type)
            .field("payloads", &self.payloads)
            .finish()
    }
}