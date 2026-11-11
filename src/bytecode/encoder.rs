// src/bytecode/encoder.rs

use crate::runtime::OpCode;

const MAGIC: &[u8; 3] = b"AVM";
const VERSION: u8 = 1;

/// Encode AVM instructions into versioned bytecode.
pub fn encode(opcodes: &[OpCode]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();

    // Bytecode header
    bytes.extend_from_slice(MAGIC);
    bytes.push(VERSION);

    for opcode in opcodes {
        match opcode {
            OpCode::Nop => {
                bytes.push(0x00);
            }

            OpCode::PushInt(value) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&value.to_le_bytes());
            }

            OpCode::PushFloat(value) => {
                bytes.push(0x02);
                bytes.extend_from_slice(&value.to_le_bytes());
            }

            OpCode::PushBool(value) => {
                bytes.push(0x03);
                bytes.push(u8::from(*value));
            }

            OpCode::PushChar(value) => {
                bytes.push(0x04);
                bytes.extend_from_slice(&(*value as u32).to_le_bytes());
            }

            OpCode::PushNull => {
                bytes.push(0x05);
            }

            OpCode::PushString(value) => {
                bytes.push(0x06);

                let string_bytes = value.as_bytes();
                let length = u32::try_from(string_bytes.len())
                    .map_err(|_| "String is too large to encode".to_string())?;

                bytes.extend_from_slice(&length.to_le_bytes());
                bytes.extend_from_slice(string_bytes);
            }

            OpCode::Print => {
                bytes.push(0x07);
            }

            OpCode::Add => {
                bytes.push(0x08);
            }
        }
    }

    Ok(bytes)
}
