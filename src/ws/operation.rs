use std::process::exit;
use crate::lua;
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

#[derive(Debug)]
pub struct Operation {
    id: String,
    op_type: OperationType,
    payloads: Vec<String>
}
impl Operation {

    // Decode the packed binary operation data.
    pub fn from_binary(bin: Vec<u8>) -> Result<Self, String> {

        let bin_size = bin.len();
        let mut pos = 0;

        // Read a string and advance position.
        fn read_str(bin: &[u8], bin_size: usize, pos: &mut usize) -> Result<String, String> {

            // Check that there is enough room for the length.
            if *pos + 4 > bin_size {
                return Err("Invalid string size for length!".to_string());
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
                return Err("Invalid size for value.".to_string());
            }

            // Read value.
            let value_bytes = &bin[*pos..end_pos];
            let out = match String::from_utf8(value_bytes.to_vec()) {
                Ok(str) => Ok(str),
                Err(_) => {
                    Err("Could not decode value utf8 bytes.".to_string())
                }
            };
            *pos = end_pos;
            out
        };
        fn read_u8(bin: &[u8], bin_size: usize, pos: &mut usize) -> Result<u8, String> {

            // Check that there is an additional byte to read.
            if *pos + 1 > bin_size {
                return Err("Invalid size for u8!".to_string())
            }

            // Read u8 (I have no idea why it has to be done like this).
            let byte = &bin[*pos];
            let result = u8::from_le_bytes([
                *byte
            ]);
            *pos += 1;
            Ok(result)
        };

        // 1. Read reply ID.
        let id = match read_str(&bin, bin_size, &mut pos) {
            Ok(reply_id) => reply_id,
            Err(_) => {
                return Err("Could not read reply ID!".to_string())
            }
        };

        // 2. Read operation type.
        let op_type = match read_u8(&bin, bin_size, &mut pos) {
            Ok(v) => match OperationType::from_u8(v) {
                None => {
                    return Err("Invalid operation type!".to_string())
                },
                Some(op_type) => op_type
            }
            Err(_) => {
                return Err("Could not read operation type!".to_string())
            }
        };

        // 3. Read payload count.
        let payload_count = match read_u8(&bin, bin_size, &mut pos) {
            Ok(payload_count) => payload_count,
            Err(_) => {
                return Err("Could not read payload count!".to_string())
            }
        };
        if payload_count > 2 {
            return Err("There cannot be more than two payload values!".to_string())
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
            Operation {
                id,
                op_type,
                payloads
            }
        )
    }

    // Actually run the operation!
    pub fn run(&mut self) -> OperationReply {

        // Short constructor.
        let replier = |success: bool, output: Option<Vec<LuaReturnValue>>| -> OperationReply {
            OperationReply::new(
                self.id.clone(),
                success,
                output
            )
        };

        return match self.op_type {

            // Join a game server.
            OperationType::ConnectToServer => {
                unimplemented!()
            },

            // Disconnect from the game server.
            OperationType::DisconnectFromServer => {
                unimplemented!()
            },

            // Execute a lua payload and return its outputs.
            OperationType::ExecuteLua => {

                // There must be two payloads here. The realm identifier and the code.
                if self.payloads.len() != 2 {
                    return replier(false, None);
                }

                // Read payload values.
                let code = match self.payloads.pop() {
                    None => {
                        return replier(false, None);
                    },
                    Some(code) => code
                };
                let realm = match self.payloads.pop() {
                    None => {
                        return replier(false, None);
                    }
                    Some(realm) => match realm.as_str() {
                        "c" => Realm::Client,
                        "m" => Realm::Menu,
                        _ => Realm::Menu
                    }
                };

                // TODO: Allow the realm to be controlled upstream.
                let payload = LuaPayload {
                    realm,
                    code
                };

                // Convert the LuaResult to an OperationReply.
                OperationReply::from(
                    lua::run(payload)
                )
            },

            // Exit the game process.
            OperationType::ExitProcess => {
                replier(true, None);
                exit(1);
            }
        }
    }
}