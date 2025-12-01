// src/parser/grammar.rs

use crate::lexer::tokenizer::{Token, TokenKind};
use crate::ast::nodes::{Program, Statement};
use crate::parser::{ParserError, ParseResult};

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
    // Example: parse a `print` statement like: print "Hello"
    if let Some(tok) = tokens.peek() {
        println!("🔍 Parsing token: {:?}", tok);
        if let TokenKind::Keyword(ref kw) = tok.kind {
            println!("🟢 Found keyword: {}", kw);
            if kw.as_str() == "print" {
                tokens.next(); // consume 'print'

                // Next token should be a string literal
                if let Some(next_tok) = tokens.next() {
                    println!("🟢 Next token: {:?}", next_tok);
                    match next_tok.kind {
                        TokenKind::StringLiteral(text) => Ok(Statement::Print(text)),
                        _ => Err(ParserError::UnexpectedToken(
                            "Expected string literal after print".into(),
                        )),
                    }
                } else {
                    Err(ParserError::UnexpectedToken(
                        "Unexpected end of input after print".into(),
                    ))
                }
            } else {
                Err(ParserError::UnexpectedToken(format!("Unknown keyword: {}", kw)))
            }
        } else {
            println!("🔴 Token is not a keyword");
            Err(ParserError::UnexpectedToken("Unknown statement".into()))
        }
    } else {
        Err(ParserError::UnexpectedToken("No more tokens".into()))
    }

}
