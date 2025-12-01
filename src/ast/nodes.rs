// src/ast/nodes.rs

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Print(String),
    // Add more: let bindings, function calls, etc.
}

#[derive(Debug)]
pub enum Expression {
    Literal(String),
    // Add math, variables, etc.
}
