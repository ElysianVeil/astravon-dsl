// src/parser/grammar.rs

use crate::ast::nodes::{Expression, Program, Statement};
use crate::lexer::tokenizer::{Token, TokenKind};
use crate::parser::{ParseResult, ParserError};

/// Parse the entire program into an AST.
pub fn parse_program(tokens: Vec<Token>) -> ParseResult<Program> {
    let mut statements = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.peek() {
        // Stop parsing at EOF
        if token.kind == TokenKind::Eof {
            break;
        }
        // This is just a placeholder; expand based on your DSL syntax.
        let stmt = parse_statement(&mut iter)?;
        statements.push(stmt);
    }

    Ok(Program { statements })
}

/// Parse a single statement.
fn parse_statement<I>(tokens: &mut std::iter::Peekable<I>) -> ParseResult<Statement>
where
    I: Iterator<Item = Token>,
{
    let token = tokens
        .next()
        .ok_or_else(|| ParserError::UnexpectedToken("Unexpected end of input".into()))?;

    match token.kind {
        TokenKind::Keyword(kw) if kw == "print" => {
            let expression = parse_expression(tokens)?;
            Ok(Statement::Print(expression))
        }

        TokenKind::Keyword(kw) => Err(ParserError::UnexpectedToken(format!(
            "Unknown keyword: {}",
            kw
        ))),

        _ => Err(ParserError::UnexpectedToken(
            "Expected a statement keyword".into(),
        )),
    }
}

fn parse_expression<I>(tokens: &mut std::iter::Peekable<I>) -> ParseResult<Expression>
where
    I: Iterator<Item = Token>,
{
    let token = tokens.next().ok_or_else(|| {
        ParserError::UnexpectedToken("Expected an expression, found end of input".into())
    })?;

    match token.kind {
        TokenKind::StringLiteral(value) => Ok(Expression::StringLiteral(value)),

        TokenKind::Integer(value) => Ok(Expression::IntegerLiteral(value)),

        TokenKind::Float(value) => Ok(Expression::FloatLiteral(value)),

        TokenKind::CharacterLiteral(value) => Ok(Expression::CharacterLiteral(value)),

        TokenKind::Keyword(value) if value == "true" => Ok(Expression::BooleanLiteral(true)),

        TokenKind::Keyword(value) if value == "false" => Ok(Expression::BooleanLiteral(false)),

        TokenKind::Keyword(value) if value == "null" => Ok(Expression::NullLiteral),

        other => Err(ParserError::UnexpectedToken(format!(
            "Expected an expression, found {:?}",
            other
        ))),
    }
}
