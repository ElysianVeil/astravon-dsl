// src/bytecode/decoder.rs

use crate::runtime::OpCode;

const MAGIC: &[u8; 3] = b"AVM";
const VERSION: u8 = 4;

/// Decode versioned AVM bytecode into instructions.
pub fn decode(bytes: &[u8]) -> Result<Vec<OpCode>, String> {
    if bytes.len() < 4 {
        return Err("Bytecode is too short to contain a valid header".into());
    }

    if &bytes[0..3] != MAGIC {
        return Err("Invalid AVM bytecode magic header".into());
    }

    if bytes[3] != VERSION {
        return Err(format!("Unsupported AVM bytecode version: {}", bytes[3]));
    }

    let mut instructions = Vec::new();
    let mut position = 4;

    while position < bytes.len() {
        let opcode = read_u8(bytes, &mut position)?;

        let instruction = match opcode {
            0x00 => OpCode::Nop,

            0x01 => OpCode::PushInt(read_i64(bytes, &mut position)?),

            0x02 => OpCode::PushFloat(read_f64(bytes, &mut position)?),

            0x03 => {
                let value = read_u8(bytes, &mut position)?;

                match value {
                    0 => OpCode::PushBool(false),
                    1 => OpCode::PushBool(true),
                    _ => {
                        return Err(format!("Invalid boolean operand: {}", value));
                    }
                }
            }

            0x04 => {
                let value = read_u32(bytes, &mut position)?;

                let character = char::from_u32(value)
                    .ok_or_else(|| format!("Invalid Unicode scalar value: {}", value))?;

                OpCode::PushChar(character)
            }

            0x05 => OpCode::PushNull,

            0x06 => OpCode::PushString(read_string(bytes, &mut position)?),

            0x07 => OpCode::Print,
            0x08 => OpCode::Add,

            0x09 => OpCode::LoadVar(read_string(bytes, &mut position)?),
            0x0A => OpCode::StoreVar(read_string(bytes, &mut position)?),

            0x0B => OpCode::Subtract,
            0x0C => OpCode::Multiply,
            0x0D => OpCode::Divide,
            0x0E => OpCode::Modulo,

            0x0F => OpCode::Equal,
            0x10 => OpCode::NotEqual,
            0x11 => OpCode::LessThan,
            0x12 => OpCode::GreaterThan,
            0x13 => OpCode::LessThanOrEqual,
            0x14 => OpCode::GreaterThanOrEqual,

            0x15 => OpCode::And,
            0x16 => OpCode::Or,

            0x17 => OpCode::Negate,
            0x18 => OpCode::Not,

            0x19 => {
                let count = read_u32(bytes, &mut position)? as usize;
                OpCode::BuildString(count)
            }

            0x20 => OpCode::EnterScope,

            0x21 => OpCode::ExitScope,

            0x22 => {
                let count = read_u32(bytes, &mut position)? as usize;
                OpCode::CreateArray(count)
            }

            0x23 => OpCode::ArrayLength,

            0x24 => OpCode::ArrayGet,

            // Unconditional control flow
            0x1A => {
                let target = read_u32(bytes, &mut position)? as usize;
                OpCode::Jump(target)
            }

            // Function calls
            0x1B => {
                let target = read_u32(bytes, &mut position)? as usize;
                let argument_count = read_u32(bytes, &mut position)? as usize;

                OpCode::Call {
                    target,
                    argument_count,
                }
            }

            0x1C => OpCode::Return,

            0x1D => OpCode::Pop,

            // Variable declarations
            0x1E => OpCode::DefineVar(read_string(bytes, &mut position)?),

            // Conditional control flow
            0x1F => {
                let target = read_u32(bytes, &mut position)? as usize;
                OpCode::JumpIfFalse(target)
            }

            unknown => {
                return Err(format!("Unknown AVM opcode: 0x{:02X}", unknown));
            }
        };

        instructions.push(instruction);
    }

    Ok(instructions)
}

// -------------------------------------------------------------
// Binary reading helpers
// -------------------------------------------------------------

fn read_u8(bytes: &[u8], position: &mut usize) -> Result<u8, String> {
    let value = *bytes
        .get(*position)
        .ok_or_else(|| "Unexpected end of bytecode while reading u8".to_string())?;

    *position += 1;

    Ok(value)
}

fn read_u32(bytes: &[u8], position: &mut usize) -> Result<u32, String> {
    let data = read_bytes(bytes, position, 4)?;

    Ok(u32::from_le_bytes(
        data.try_into()
            .map_err(|_| "Invalid u32 operand".to_string())?,
    ))
}

fn read_i64(bytes: &[u8], position: &mut usize) -> Result<i64, String> {
    let data = read_bytes(bytes, position, 8)?;

    Ok(i64::from_le_bytes(
        data.try_into()
            .map_err(|_| "Invalid i64 operand".to_string())?,
    ))
}

fn read_f64(bytes: &[u8], position: &mut usize) -> Result<f64, String> {
    let data = read_bytes(bytes, position, 8)?;

    Ok(f64::from_le_bytes(
        data.try_into()
            .map_err(|_| "Invalid f64 operand".to_string())?,
    ))
}

fn read_bytes<'a>(
    bytes: &'a [u8],
    position: &mut usize,
    length: usize,
) -> Result<&'a [u8], String> {
    let end = position
        .checked_add(length)
        .ok_or_else(|| "Bytecode position overflow".to_string())?;

    let data = bytes
        .get(*position..end)
        .ok_or_else(|| "Unexpected end of bytecode while reading operand".to_string())?;

    *position = end;

    Ok(data)
}

fn read_string(bytes: &[u8], position: &mut usize) -> Result<String, String> {
    let length = read_u32(bytes, position)? as usize;
    let data = read_bytes(bytes, position, length)?;

    String::from_utf8(data.to_vec()).map_err(|_| "Invalid UTF-8 in string bytecode".to_string())
}
