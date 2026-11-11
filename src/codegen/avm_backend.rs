// src/codegen/avm_backend.rs

use crate::ast::nodes::{Expression, Program, Statement};
use crate::runtime::OpCode;

/// Generate AVM instructions from an AstravonDSL AST.
pub fn compile_program(program: &Program) -> Result<Vec<OpCode>, String> {
    let mut instructions = Vec::new();

    for statement in &program.statements {
        compile_statement(statement, &mut instructions)?;
    }

    Ok(instructions)
}

fn compile_statement(statement: &Statement, instructions: &mut Vec<OpCode>) -> Result<(), String> {
    match statement {
        Statement::Print(expression) => {
            compile_expression(expression, instructions)?;
            instructions.push(OpCode::Print);
        }
    }

    Ok(())
}

fn compile_expression(
    expression: &Expression,
    instructions: &mut Vec<OpCode>,
) -> Result<(), String> {
    match expression {
        Expression::StringLiteral(value) => {
            instructions.push(OpCode::PushString(value.clone()));
        }

        Expression::IntegerLiteral(value) => {
            instructions.push(OpCode::PushInt(*value));
        }

        Expression::FloatLiteral(value) => {
            instructions.push(OpCode::PushFloat(*value));
        }

        Expression::BooleanLiteral(value) => {
            instructions.push(OpCode::PushBool(*value));
        }

        Expression::CharacterLiteral(value) => {
            instructions.push(OpCode::PushChar(*value));
        }

        Expression::NullLiteral => {
            instructions.push(OpCode::PushNull);
        }
    }

    Ok(())
}
