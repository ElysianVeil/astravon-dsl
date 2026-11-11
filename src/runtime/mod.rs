// src/runtime/mod.rs
pub mod executor;

use std::collections::HashMap;
pub type NativeFn = fn(Vec<Value>) -> Value;

#[derive(Debug, Clone)]
pub enum OpCode {
    Nop,

    PushInt(i64),
    PushFloat(f64),
    PushBool(bool),
    PushChar(char),
    PushNull,
    PushString(String),

    Print,
    Add,
    // … add more as your DSL grows
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: OpCode,
    pub args: Vec<Value>,
}

pub struct AVM {
    pub instructions: Vec<Instruction>,
    pub stack: Vec<Value>,
    pub native_functions: HashMap<String, NativeFn>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(String),
    Null,
    // later: AI objects, blockchain handles, DNA sequences, etc.
}

impl AVM {
    pub fn new(opcodes: Vec<OpCode>) -> Self {
        let instructions: Vec<Instruction> = opcodes
            .into_iter()
            .map(|op| Instruction {
                opcode: op,
                args: Vec::new(),
            })
            .collect();

        Self {
            instructions,
            stack: Vec::new(),
            native_functions: HashMap::new(),
        }
    }

    pub fn load(&mut self, instructions: Vec<Instruction>) {
        self.instructions = instructions;
    }

    pub fn register_native(&mut self, name: &str, func: NativeFn) {
        self.native_functions.insert(name.to_string(), func);
    }
}
