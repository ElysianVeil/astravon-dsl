// src/semantics/analyzer.rs

use crate::ast::nodes::{Program, Statement, Expression};
use std::collections::HashMap;

/// Represents the type of a value in your DSL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    String,
    Int,
    Bool,
    Void,
    Unknown,
}

/// Represents a symbol (variable or function) in scope.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
}

/// The semantic analyzer’s state (symbol table, etc.)
pub struct Analyzer {
    pub symbols: HashMap<String, Symbol>,
}

#[derive(Debug)]
pub enum SemanticError {
    UndeclaredVariable(String),
    TypeMismatch { expected: Type, found: Type },
}

/// The result type for semantic analysis.
pub type SemanticResult<T> = Result<T, SemanticError>;

impl Analyzer {
    /// Create a new Analyzer with an empty symbol table.
    pub fn new() -> Self {
        Analyzer {
            symbols: HashMap::new(),
        }
    }

    /// Entry point: analyze a whole program AST.
    pub fn analyze_program(&mut self, program: &Program) -> SemanticResult<()> {
        for stmt in &program.statements {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> SemanticResult<()> {
        match stmt {
            Statement::Print(expr) => {
                let ty = self.analyze_expression(&Expression::Literal(expr.clone()))?;
                // Print can take any type, but you could restrict it:
                if ty != Type::String {
                    return Err(SemanticError::TypeMismatch {
                        expected: Type::String,
                        found: ty,
                    });
                }
                Ok(())
            }
            // Add other statement kinds here
        }
    }

    fn analyze_expression(&mut self, expr: &Expression) -> SemanticResult<Type> {
        match expr {
            Expression::Literal(lit) => {
                // crude: if literal looks numeric, return Int; else String
                if lit.parse::<i64>().is_ok() {
                    Ok(Type::Int)
                } else if lit.parse::<f64>().is_ok() {
                    Ok(Type::Int) // Or create a Float type if you like
                } else {
                    Ok(Type::String)
                }

            }
            // Add more: variable references, math, function calls
        }
    }
}
