// src/codegen/wasm_backend.rs

use crate::ast::nodes::{Program, Statement};

pub struct WasmBackend {}

impl WasmBackend {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<Vec<u8>, String> {
        let mut wasm_bytes = Vec::new();

        for statement in &program.statements {
            self.compile_statement(statement, &mut wasm_bytes)?;
        }

        Ok(wasm_bytes)
    }

    fn compile_statement(
        &mut self,
        statement: &Statement,
        _wasm_bytes: &mut Vec<u8>,
    ) -> Result<(), String> {
        match statement {
            Statement::Print(_) => Err("WASM print compilation is not implemented yet".to_string()),

            Statement::Let { .. } | Statement::Assignment { .. } => {
                Err("WASM variable compilation is not implemented yet".to_string())
            }

            Statement::If { .. } => {
                Err("WASM conditional compilation is not implemented yet".to_string())
            }

            Statement::While { .. } => {
                Err("WASM while loop compilation is not implemented yet".to_string())
            }

            Statement::For { .. } => {
                Err("WASM for loop compilation is not implemented yet".to_string())
            }

            Statement::Break => Err("WASM break compilation is not implemented yet".to_string()),

            Statement::Continue => {
                Err("WASM continue compilation is not implemented yet".to_string())
            }

            Statement::Function { name, .. } => Err(format!(
                "WASM function compilation is not implemented yet: {}",
                name
            )),

            Statement::Return(_) => {
                Err("WASM return compilation is not implemented yet".to_string())
            }

            Statement::Expression(_) => {
                Err("WASM expression-statement compilation is not implemented yet".to_string())
            }
        }
    }
}
