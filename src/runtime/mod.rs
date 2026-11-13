// src/runtime/mod.rs

pub mod executor;

use std::collections::HashMap;

pub type NativeFn = fn(Vec<Value>) -> Value;

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    Nop,

    // Values
    PushInt(i64),
    PushFloat(f64),
    PushBool(bool),
    PushChar(char),
    PushNull,
    PushString(String),
    BuildString(usize),

    // Variables
    LoadVar(String),
    DefineVar(String),
    StoreVar(String),

    // Control flow and functions
    Jump(usize),
    JumpIfFalse(usize),
    Call {
        target: usize,
        argument_count: usize,
    },
    Return,
    Pop,

    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparisons
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,

    // Logical and unary operations
    And,
    Or,
    Negate,
    Not,

    // Output
    Print,

    // Scope management
    EnterScope,
    ExitScope,

    CreateArray(usize),
    ArrayLength,
    ArrayGet,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: OpCode,
    pub args: Vec<Value>,
}

pub struct AVM {
    pub instructions: Vec<Instruction>,
    pub stack: Vec<Value>,
    pub variables: HashMap<String, Value>,
    pub native_functions: HashMap<String, NativeFn>,
    pub scopes: Vec<HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(String),
    Array(Vec<Value>),
    Null,
}

impl AVM {
    pub fn new(opcodes: Vec<OpCode>) -> Self {
        let instructions = opcodes
            .into_iter()
            .map(|opcode| Instruction {
                opcode,
                args: Vec::new(),
            })
            .collect();

        Self {
            instructions,
            stack: Vec::new(),
            variables: HashMap::new(),
            scopes: Vec::new(),
            native_functions: HashMap::new(),
        }
    }

    pub fn load(&mut self, instructions: Vec<Instruction>) {
        self.instructions = instructions;
        self.stack.clear();
        self.variables.clear();
        self.scopes.clear();
    }

    pub fn register_native(&mut self, name: &str, func: NativeFn) {
        self.native_functions.insert(name.to_string(), func);
    }
}
