// src/runtime/executor.rs

use crate::runtime::{AVM, OpCode, Value};

pub fn execute(vm: &mut AVM, _bytecode: Vec<u8>) -> Result<(), String> {
    let mut ip = 0;

    while ip < vm.instructions.len() {
        let instruction = vm.instructions[ip].clone();

        match instruction.opcode {
            OpCode::Nop => {}

            OpCode::PushString(value) => {
                vm.stack.push(Value::Str(value));
            }

            OpCode::PushInt(value) => {
                vm.stack.push(Value::Int(value));
            }

            OpCode::PushFloat(value) => {
                vm.stack.push(Value::Float(value));
            }

            OpCode::PushBool(value) => {
                vm.stack.push(Value::Bool(value));
            }

            OpCode::PushChar(value) => {
                vm.stack.push(Value::Char(value));
            }

            OpCode::PushNull => {
                vm.stack.push(Value::Null);
            }

            OpCode::Add => {
                if vm.stack.len() < 2 {
                    return Err(format!("Stack underflow on Add at ip={}", ip));
                }

                let b = vm.stack.pop().unwrap();
                let a = vm.stack.pop().unwrap();

                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        let result = a
                            .checked_add(b)
                            .ok_or_else(|| format!("Integer overflow on Add at ip={}", ip))?;

                        vm.stack.push(Value::Int(result));
                    }

                    (Value::Float(a), Value::Float(b)) => {
                        vm.stack.push(Value::Float(a + b));
                    }

                    (a, b) => {
                        return Err(format!("Type error on Add at ip={}: {:?} + {:?}", ip, a, b));
                    }
                }
            }

            OpCode::Print => {
                let value = vm
                    .stack
                    .pop()
                    .ok_or_else(|| format!("Stack underflow on Print at ip={}", ip))?;

                match value {
                    Value::Str(value) => println!("{}", value),
                    Value::Int(value) => println!("{}", value),
                    Value::Float(value) => println!("{}", value),
                    Value::Bool(value) => println!("{}", value),
                    Value::Char(value) => println!("{}", value),
                    Value::Null => println!("null"),
                }
            }
        }

        // Advance after successfully executing the current instruction.
        ip += 1;
    }

    Ok(())
}
