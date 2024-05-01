use std::fmt;
use std::fmt::Formatter;
use crate::lua::types::{LuaResult, LuaReturnValue};
use crate::ws::binary::{BinaryBuffer, BinaryWriter};

/*

    Reply message structure:
        reply_id - ( size: u32, utf: [u8]*size )
        success - 1 byte ( 0 / 1 )
        output_exists - 1 byte ( 0 / 1 )

        (ONLY IF output_exists = 1):
            count - u32

            (FOR count):
                type_id - u32
                value - .... (controlled by type)

 */

pub struct OperationReply {
    id: String,
    success: bool,
    output: Option<Vec<LuaReturnValue>>,
    error_message: Option<String>
}
impl OperationReply {
    pub fn new(id: String, success: bool, output: Option<Vec<LuaReturnValue>>, error_message: Option<String>) -> OperationReply {
        OperationReply {
            id,
            success,
            output,
            error_message
        }
    }
}
impl BinaryWriter for OperationReply {
    fn write_binary(&self, buffer: &mut BinaryBuffer) -> Result<(), &str> {

        // Write reply ID.
        buffer.write_str(&self.id.as_str());

        // Write success state.
        buffer.write_bool(self.success);

        // If unsuccessful, write the error message.
        if !self.success {

            // Write error message.
            buffer.write_bool(self.error_message.is_some());
            if let Some(error_message) = &self.error_message {
                buffer.write_str(error_message.as_str());
            }

        } else {

            // Write output (where first byte specifies if it actually exists).
            buffer.write_bool(self.output.is_some());
            if let Some(output) = &self.output {

                // Write all LuaReturnValue's.
                buffer.write_u32(output.len() as u32);
                for v in output {
                    if let Err(e) = v.write_binary(buffer) {
                        return Err(e)
                    }
                }
            }
        }
        Ok(())
    }
}
impl From<(String, LuaResult)> for OperationReply {
    fn from(data: (String, LuaResult)) -> Self {
        return OperationReply::new(
            data.0,
            data.1.success,
            data.1.output,
            data.1.error_message
        );
    }
}
impl fmt::Debug for OperationReply {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("OperationReply");

        // Add default fields.
        debug_struct.field("id", &self.id)
            .field("success", &self.success);

        // Only add output fields if there are some.
        if let Some(output) = &self.output {
            for (i, value) in output.iter().enumerate() {
                debug_struct.field(format!("output-{i}").as_str(), value);
            }
        }
        debug_struct.finish()
    }
}