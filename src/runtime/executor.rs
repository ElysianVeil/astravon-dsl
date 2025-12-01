// src/runtime/executor.rs
use crate::runtime::{AVM, OpCode, Value};

pub fn execute(vm: &mut AVM, _bytecode: Vec<u8>) -> Result<(), String> {
    let mut ip = 0; // instruction pointer
    while ip < vm.instructions.len() {
        let instr = &vm.instructions[ip];
        match &instr.opcode {
            OpCode::Nop => {
                ip += 1;
            }
            OpCode::PushString(s) => vm.stack.push(Value::Str(s.to_string())),
            OpCode::PushInt(i) => {
                vm.stack.push(Value::Int(*i));
                ip += 1;
            }
            OpCode::PushStr(s) => {
                vm.stack.push(Value::Str(s.to_string().clone()));
                ip += 1;
            }
            OpCode::Add => {
                // Check stack before popping
                if vm.stack.len() < 2 {
                    return Err(format!("Stack underflow on Add at ip={}", ip));
                }
                let b = vm.stack.pop().unwrap();
                let a = vm.stack.pop().unwrap();
                if let (Value::Int(ai), Value::Int(bi)) = (a.clone(), b.clone()) {
                    vm.stack.push(Value::Int(ai + bi));
                } else {
                    return Err(format!("Type error on Add at ip={}: {:?}, {:?}",ip, a, b));
                }
                ip += 1;
                
            }
            OpCode::Print => {
                if let Some(v) = vm.stack.pop() {
                    match v {
                        Value::Str(ref s) => println!("{}", s),
                        other => println!("{:?}", other),
                    }
                } else {
                    eprintln!("Stack underflow on Print");
                }
                ip += 1;
            }
            // add your AI, blockchain, DNA, etc. opcodes here later
        }
    }
    Ok(())
}
