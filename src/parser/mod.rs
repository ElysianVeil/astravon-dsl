// src/parser/mod.rs

pub mod grammar;

use crate::lexer::tokenizer::Token;

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(String),
    EndOfInput,
}

pub type ParseResult<T> = Result<T, ParserError>;

/// Parse a complete AstravonDSL program.
pub fn parse(tokens: Vec<Token>) -> ParseResult<crate::ast::nodes::Program> {
    grammar::parse_program(tokens)
}
