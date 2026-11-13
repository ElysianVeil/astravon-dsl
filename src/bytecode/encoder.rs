// src/bytecode/encoder.rs

use crate::runtime::OpCode;

const MAGIC: &[u8; 3] = b"AVM";
const VERSION: u8 = 4;

/// Encode AVM instructions into versioned bytecode.
pub fn encode(opcodes: &[OpCode]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();

    // Bytecode header
    bytes.extend_from_slice(MAGIC);
    bytes.push(VERSION);

    for opcode in opcodes {
        match opcode {
            OpCode::Nop => bytes.push(0x00),

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

            OpCode::PushNull => bytes.push(0x05),

            OpCode::PushString(value) => {
                bytes.push(0x06);
                write_string(&mut bytes, value)?;
            }

            OpCode::Print => bytes.push(0x07),
            OpCode::Add => bytes.push(0x08),

            OpCode::LoadVar(name) => {
                bytes.push(0x09);
                write_string(&mut bytes, name)?;
            }

            OpCode::StoreVar(name) => {
                bytes.push(0x0A);
                write_string(&mut bytes, name)?;
            }

            OpCode::Subtract => bytes.push(0x0B),
            OpCode::Multiply => bytes.push(0x0C),
            OpCode::Divide => bytes.push(0x0D),
            OpCode::Modulo => bytes.push(0x0E),

            OpCode::Equal => bytes.push(0x0F),
            OpCode::NotEqual => bytes.push(0x10),
            OpCode::LessThan => bytes.push(0x11),
            OpCode::GreaterThan => bytes.push(0x12),
            OpCode::LessThanOrEqual => bytes.push(0x13),
            OpCode::GreaterThanOrEqual => bytes.push(0x14),

            OpCode::And => bytes.push(0x15),
            OpCode::Or => bytes.push(0x16),

            OpCode::Negate => bytes.push(0x17),
            OpCode::Not => bytes.push(0x18),

            OpCode::BuildString(count) => {
                bytes.push(0x19);

                let count = u32::try_from(*count)
                    .map_err(|_| "Too many string segments to encode".to_string())?;

                bytes.extend_from_slice(&count.to_le_bytes());
            }

            OpCode::CreateArray(count) => {
                bytes.push(0x22);

                let count = u32::try_from(*count)
                    .map_err(|_| "Too many array elements to encode".to_string())?;

                bytes.extend_from_slice(&count.to_le_bytes());
            }

            OpCode::ArrayLength => bytes.push(0x23),

            OpCode::ArrayGet => bytes.push(0x24),

            // Unconditional control flow
            OpCode::Jump(target) => {
                bytes.push(0x1A);
                write_usize(&mut bytes, *target, "Jump target")?;
            }

            // Function calls
            OpCode::Call {
                target,
                argument_count,
            } => {
                bytes.push(0x1B);

                write_usize(&mut bytes, *target, "Call target")?;
                write_usize(&mut bytes, *argument_count, "Function argument count")?;
            }

            OpCode::Return => bytes.push(0x1C),

            OpCode::Pop => bytes.push(0x1D),

            // Variable declarations
            OpCode::DefineVar(name) => {
                bytes.push(0x1E);
                write_string(&mut bytes, name)?;
            }

            // Conditional control flow
            OpCode::JumpIfFalse(target) => {
                bytes.push(0x1F);
                write_usize(&mut bytes, *target, "Conditional jump target")?;
            }

            OpCode::EnterScope => bytes.push(0x20),

            OpCode::ExitScope => bytes.push(0x21),
        }
    }

    Ok(bytes)
}

/// Encode a usize operand as a u32.
fn write_usize(bytes: &mut Vec<u8>, value: usize, description: &str) -> Result<(), String> {
    let value =
        u32::try_from(value).map_err(|_| format!("{} is too large to encode", description))?;

    bytes.extend_from_slice(&value.to_le_bytes());

    Ok(())
}

/// Encode a UTF-8 string with a u32 length prefix.
fn write_string(bytes: &mut Vec<u8>, value: &str) -> Result<(), String> {
    let string_bytes = value.as_bytes();

    let length = u32::try_from(string_bytes.len())
        .map_err(|_| "String is too large to encode".to_string())?;

    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(string_bytes);

    Ok(())
}
