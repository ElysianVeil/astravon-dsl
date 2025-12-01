// src/codegen/wasm_backend.rs

use crate::ast::nodes::{Program, Statement};

pub struct WasmBackend {
    // you can store a Wasm module builder here (like wasm_encoder)
}

impl WasmBackend {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile_program(&mut self, program: &Program) -> Vec<u8> {
        // generate wasm bytecode into a Vec<u8>
        let mut wasm_bytes = Vec::new();

        for stmt in &program.statements {
            self.compile_statement(stmt, &mut wasm_bytes);
        }

        wasm_bytes
    }

    fn compile_statement(&mut self, stmt: &Statement, _wasm_bytes: &mut Vec<u8>) {
        match stmt {
            Statement::Print(_expr) => {
                // encode WASM instructions to print string
            }
            // ...
        }
    }
}
