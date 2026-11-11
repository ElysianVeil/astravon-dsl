// src/bytecode/decoder.rs

use crate::runtime::OpCode;

const MAGIC: &[u8; 3] = b"AVM";
const VERSION: u8 = 1;

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

            0x01 => {
                let value = read_i64(bytes, &mut position)?;
                OpCode::PushInt(value)
            }

            0x02 => {
                let value = read_f64(bytes, &mut position)?;
                OpCode::PushFloat(value)
            }

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

            0x06 => {
                let length = read_u32(bytes, &mut position)? as usize;

                let end = position
                    .checked_add(length)
                    .ok_or_else(|| "String length overflow in bytecode".to_string())?;

                let string_data = bytes
                    .get(position..end)
                    .ok_or_else(|| "Truncated string data in bytecode".to_string())?;

                let value = String::from_utf8(string_data.to_vec())
                    .map_err(|_| "Invalid UTF-8 in string bytecode".to_string())?;

                position = end;

                OpCode::PushString(value)
            }

            0x07 => OpCode::Print,

            0x08 => OpCode::Add,

            unknown => {
                return Err(format!("Unknown AVM opcode: 0x{:02X}", unknown));
            }
        };

        instructions.push(instruction);
    }

    Ok(instructions)
}

fn read_u8(bytes: &[u8], position: &mut usize) -> Result<u8, String> {
    let value = *bytes
        .get(*position)
        .ok_or_else(|| "Unexpected end of bytecode while reading u8".to_string())?;

    *position += 1;
    Ok(value)
}

fn read_u32(bytes: &[u8], position: &mut usize) -> Result<u32, String> {
    let end = position
        .checked_add(4)
        .ok_or_else(|| "Bytecode position overflow".to_string())?;

    let data = bytes
        .get(*position..end)
        .ok_or_else(|| "Unexpected end of bytecode while reading u32".to_string())?;

    *position = end;

    Ok(u32::from_le_bytes(data.try_into().unwrap()))
}

fn read_i64(bytes: &[u8], position: &mut usize) -> Result<i64, String> {
    let end = position
        .checked_add(8)
        .ok_or_else(|| "Bytecode position overflow".to_string())?;

    let data = bytes
        .get(*position..end)
        .ok_or_else(|| "Unexpected end of bytecode while reading i64".to_string())?;

    *position = end;

    Ok(i64::from_le_bytes(data.try_into().unwrap()))
}

fn read_f64(bytes: &[u8], position: &mut usize) -> Result<f64, String> {
    let end = position
        .checked_add(8)
        .ok_or_else(|| "Bytecode position overflow".to_string())?;

    let data = bytes
        .get(*position..end)
        .ok_or_else(|| "Unexpected end of bytecode while reading f64".to_string())?;

    *position = end;

    Ok(f64::from_le_bytes(data.try_into().unwrap()))
}
