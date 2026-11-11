// src/parser/mod.rs

pub mod grammar;

use crate::lexer::tokenizer::Token; // assuming tokenizer.rs defines Token
use grammar::parse_program;

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(String),
    EndOfInput,
}

pub type ParseResult<T> = Result<T, ParserError>;

/// Main parser interface — turns a vector of tokens into an AST.
pub fn parse(tokens: Vec<Token>) -> ParseResult<crate::ast::nodes::Program> {
    parse_program(tokens)
}
