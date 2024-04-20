use crate::lua::types::{LuaResult, LuaReturnValue};

/*

    Reply message structure:
        reply_id - ( size: u32, utf: [u8]*size )
        success - 1 byte ( 0 / 1 )
        output_exists - 1 byte ( 0 / 1 )

        (ONLY IF output_exists = 1):
            type_id - u32
            value - .... (controlled by type)

 */

pub struct OperationReply {
    id: String,
    success: bool,
    output: Option<Vec<LuaReturnValue>>
}
impl OperationReply {
    pub fn new(id: String, success: bool, output: Option<Vec<LuaReturnValue>>) -> OperationReply {
        OperationReply {
            id,
            success,
            output
        }
    }

    // Convert the reply to binary.
    pub fn to_binary(&self) -> Vec<u8> {

        let mut encoded: Vec<u8> = Vec::new();
        let add_str = |encoded: &mut Vec<u8>, v: &str| -> () {

            let utf8_bytes = v.as_bytes();
            let length = utf8_bytes.len() as u32;

            encoded.extend_from_slice(&length.to_le_bytes()); // String size
            encoded.extend_from_slice(utf8_bytes); // String value
        };
        let add_bool = |encoded: &mut Vec<u8>, v: bool| -> () {
            encoded.push(if v { 0x01 } else { 0x00 });
        };

        // Write reply ID.
        add_str(&mut encoded, &self.id.as_str());

        // Write success state.
        add_bool(&mut encoded, self.success);

        // Write output (where first byte specifies if it actually exists).
        add_bool(&mut encoded, self.output.is_some());
        if let Some(output) = &self.output {

            // Write all LuaReturnValue's.
            encoded.extend_from_slice(&(output.len() as u32).to_le_bytes());
            for v in output {
                v.write_binary(&mut encoded);
            }
        }

        encoded
    }
}
impl From<LuaResult> for OperationReply {
    fn from(result: LuaResult) -> Self {
        return OperationReply::new(String::new(), result.success, result.output);
    }
}