// src/semantics/analyzer.rs

use crate::ast::nodes::{Expression, Program, Statement};
use std::collections::HashMap;

/// Represents the type of a value in AstravonDSL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    String,
    Int,
    Float,
    Bool,
    Char,
    Null,
    Void,
    Unknown,
}

/// Represents a variable or function in the symbol table.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
}

/// Semantic analyzer state.
pub struct Analyzer {
    pub symbols: HashMap<String, Symbol>,
}

#[derive(Debug)]
pub enum SemanticError {
    UndeclaredVariable(String),

    TypeMismatch { expected: Type, found: Type },

    UnsupportedExpression(String),
}

pub type SemanticResult<T> = Result<T, SemanticError>;

impl Analyzer {
    /// Create a new semantic analyzer.
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    /// Analyze an entire program.
    pub fn analyze_program(&mut self, program: &Program) -> SemanticResult<()> {
        for statement in &program.statements {
            self.analyze_statement(statement)?;
        }

        Ok(())
    }

    /// Analyze an individual statement.
    fn analyze_statement(&mut self, statement: &Statement) -> SemanticResult<()> {
        match statement {
            Statement::Print(expression) => {
                let _ty = self.analyze_expression(expression)?;

                // All currently supported primitive values are printable.
                Ok(())
            }
        }
    }

    /// Determine the semantic type of an expression.
    fn analyze_expression(&mut self, expression: &Expression) -> SemanticResult<Type> {
        match expression {
            Expression::StringLiteral(_) => Ok(Type::String),

            Expression::IntegerLiteral(_) => Ok(Type::Int),

            Expression::FloatLiteral(_) => Ok(Type::Float),

            Expression::BooleanLiteral(_) => Ok(Type::Bool),

            Expression::CharacterLiteral(_) => Ok(Type::Char),

            Expression::NullLiteral => Ok(Type::Null),
        }
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}
