// src/parser/grammar.rs

use crate::ast::nodes::{
    BinaryOperator, Expression, MatchArm, MatchPattern, Parameter, Program, Statement,
    StringSegment, TypeAnnotation, UnaryOperator,
};

use crate::lexer::tokenizer::{Lexer, StringPart, Token, TokenKind};
use crate::parser::{ParseResult, ParserError};

type TokenStream = std::iter::Peekable<std::vec::IntoIter<Token>>;

/// Parse an entire Astravon program.
pub fn parse_program(tokens: Vec<Token>) -> ParseResult<Program> {
    let mut tokens = tokens.into_iter().peekable();
    let mut statements = Vec::new();

    skip_statement_separators(&mut tokens);

    while !matches!(
        tokens.peek().map(|token| &token.kind),
        Some(TokenKind::Eof) | None
    ) {
        let statement = parse_statement(&mut tokens)?;
        statements.push(statement);

        skip_statement_separators(&mut tokens);
    }

    Ok(Program { statements })
}

/// Parse a standalone expression.
pub fn parse_expression(tokens: Vec<Token>) -> ParseResult<Expression> {
    let mut tokens = tokens.into_iter().peekable();

    skip_statement_separators(&mut tokens);

    let expression = parse_expression_bp(&mut tokens, 0)?;

    skip_statement_separators(&mut tokens);

    if !matches!(
        tokens.peek().map(|token| &token.kind),
        Some(TokenKind::Eof) | None
    ) {
        return Err(ParserError::UnexpectedToken(
            "Unexpected tokens after expression".into(),
        ));
    }

    Ok(expression)
}

/// Parse one statement.
fn parse_statement(tokens: &mut TokenStream) -> ParseResult<Statement> {
    let token = tokens
        .next()
        .ok_or_else(|| ParserError::UnexpectedToken("Unexpected end of input".into()))?;

    match token.kind {
        TokenKind::Let => parse_let_statement(tokens),

        TokenKind::Print => {
            let expression = parse_expression_bp(tokens, 0)?;
            Ok(Statement::Print(expression))
        }

        TokenKind::Fn => parse_function_statement(tokens),

        TokenKind::Return => parse_return_statement(tokens),

        TokenKind::If => parse_if_statement(tokens),

        TokenKind::While => parse_while_statement(tokens),

        TokenKind::For => parse_for_statement(tokens),

        TokenKind::Break => Ok(Statement::Break),

        TokenKind::Continue => Ok(Statement::Continue),

        TokenKind::Identifier(name) => match tokens.peek().map(|token| &token.kind) {
            Some(TokenKind::Operator(operator)) if operator == "=" => {
                tokens.next();

                let value = parse_expression_bp(tokens, 0)?;

                Ok(Statement::Assignment { name, value })
            }

            _ => {
                let expression =
                    parse_expression_bp_from_left(Expression::Identifier(name), tokens, 0)?;

                match expression {
                    Expression::Call { .. } => Ok(Statement::Expression(expression)),

                    _ => Err(ParserError::UnexpectedToken(
                        "Only function calls are allowed as standalone expressions".into(),
                    )),
                }
            }
        },

        other => Err(ParserError::UnexpectedToken(format!(
            "Unexpected token at statement start: {:?}",
            other
        ))),
    }
}

/// Parse a function declaration:
///
/// fn add(a: Int, b: Int) -> Int {
///     return a + b;
/// }
fn parse_function_statement(tokens: &mut TokenStream) -> ParseResult<Statement> {
    let name = match tokens.next() {
        Some(Token {
            kind: TokenKind::Identifier(name),
            ..
        }) => name,

        _ => {
            return Err(ParserError::UnexpectedToken(
                "Expected function name after 'fn'".into(),
            ));
        }
    };

    expect_symbol(tokens, '(')?;

    let mut parameters = Vec::new();

    skip_newlines(tokens);

    if !matches!(
        tokens.peek().map(|token| &token.kind),
        Some(TokenKind::Symbol(')'))
    ) {
        loop {
            let parameter_name = match tokens.next() {
                Some(Token {
                    kind: TokenKind::Identifier(name),
                    ..
                }) => name,

                _ => {
                    return Err(ParserError::UnexpectedToken(
                        "Expected parameter name".into(),
                    ));
                }
            };

            expect_symbol(tokens, ':')?;

            let type_name = match tokens.next() {
                Some(Token {
                    kind: TokenKind::Identifier(name),
                    ..
                }) => name,

                _ => {
                    return Err(ParserError::UnexpectedToken(
                        "Expected parameter type".into(),
                    ));
                }
            };

            let type_annotation = parse_type_annotation(&type_name)?;

            parameters.push(Parameter {
                name: parameter_name,
                type_annotation,
            });

            skip_newlines(tokens);

            match tokens.peek().map(|token| &token.kind) {
                Some(TokenKind::Symbol(',')) => {
                    tokens.next();
                    skip_newlines(tokens);
                }

                Some(TokenKind::Symbol(')')) => break,

                other => {
                    return Err(ParserError::UnexpectedToken(format!(
                        "Expected ',' or ')' after function parameter, found {:?}",
                        other
                    )));
                }
            }
        }
    }

    expect_symbol(tokens, ')')?;

    // Optional return type: -> Type
    let return_type = match tokens.peek().map(|token| &token.kind) {
        Some(TokenKind::Operator(operator)) if operator == "-" => {
            tokens.next();

            match tokens.next() {
                Some(Token {
                    kind: TokenKind::Operator(operator),
                    ..
                }) if operator == ">" => {}

                _ => {
                    return Err(ParserError::UnexpectedToken(
                        "Expected '>' after '-' in return type".into(),
                    ));
                }
            }

            let type_name = match tokens.next() {
                Some(Token {
                    kind: TokenKind::Identifier(name),
                    ..
                }) => name,

                _ => {
                    return Err(ParserError::UnexpectedToken(
                        "Expected return type after '->'".into(),
                    ));
                }
            };

            Some(parse_type_annotation(&type_name)?)
        }

        _ => None,
    };

    let body = parse_block(tokens)?;

    Ok(Statement::Function {
        name,
        parameters,
        return_type,
        body,
    })
}

/// Parse a function or other block body.
fn parse_block(tokens: &mut TokenStream) -> ParseResult<Vec<Statement>> {
    expect_symbol(tokens, '{')?;

    let mut statements = Vec::new();

    skip_statement_separators(tokens);

    loop {
        match tokens.peek().map(|token| &token.kind) {
            Some(TokenKind::Symbol('}')) => {
                tokens.next();
                break;
            }

            Some(TokenKind::Eof) | None => {
                return Err(ParserError::UnexpectedToken(
                    "Unterminated block: expected '}'".into(),
                ));
            }

            _ => {
                let statement = parse_statement(tokens)?;
                statements.push(statement);

                skip_statement_separators(tokens);
            }
        }
    }

    Ok(statements)
}

/// Parse:
///
/// return;
/// return expression;
fn parse_return_statement(tokens: &mut TokenStream) -> ParseResult<Statement> {
    let has_expression = !matches!(
        tokens.peek().map(|token| &token.kind),
        Some(TokenKind::Newline)
            | Some(TokenKind::Symbol(';'))
            | Some(TokenKind::Symbol('}'))
            | Some(TokenKind::Eof)
            | None
    );

    if has_expression {
        let expression = parse_expression_bp(tokens, 0)?;
        Ok(Statement::Return(Some(expression)))
    } else {
        Ok(Statement::Return(None))
    }
}

/// Parse conditional statements:
///
/// if condition {
///     statements
/// } else {
///     statements
/// }
///
/// Also supports:
///
/// if condition {
///     ...
/// } else if another_condition {
///     ...
/// }
fn parse_if_statement(tokens: &mut TokenStream) -> ParseResult<Statement> {
    // Parse the condition.
    let condition = parse_expression_bp(tokens, 0)?;

    // Permit a newline between the condition and opening brace.
    skip_newlines(tokens);

    // Parse the true branch.
    let then_branch = parse_block(tokens)?;

    // Permit an else on the next line.
    skip_newlines(tokens);

    // Check for an optional else branch.
    let else_branch = match tokens.peek().map(|token| &token.kind) {
        Some(TokenKind::Else) => {
            tokens.next();

            skip_newlines(tokens);

            match tokens.peek().map(|token| &token.kind) {
                // Handle: else if condition { ... }
                Some(TokenKind::If) => {
                    tokens.next();

                    let nested_if = parse_if_statement(tokens)?;

                    Some(vec![nested_if])
                }

                // Handle: else { ... }
                Some(TokenKind::Symbol('{')) => Some(parse_block(tokens)?),

                other => {
                    return Err(ParserError::UnexpectedToken(format!(
                        "Expected 'if' or '{{' after 'else', found {:?}",
                        other
                    )));
                }
            }
        }

        _ => None,
    };

    Ok(Statement::If {
        condition,
        then_branch,
        else_branch,
    })
}

/// Parse a while-loop statement:
///
/// while condition {
///     statements
/// }
fn parse_while_statement(tokens: &mut TokenStream) -> ParseResult<Statement> {
    // Parse the loop condition.
    let condition = parse_expression_bp(tokens, 0)?;

    // Permit a newline between the condition and opening brace.
    skip_newlines(tokens);

    // Parse the loop body.
    let body = parse_block(tokens)?;

    Ok(Statement::While { condition, body })
}

/// Parse a for-in loop.
///
/// for item in collection {
///     statements
/// }
fn parse_for_statement(tokens: &mut TokenStream) -> ParseResult<Statement> {
    let variable = match tokens.next() {
        Some(token) => match token.kind {
            TokenKind::Identifier(name) => name,
            _ => {
                return Err(ParserError::UnexpectedToken(format!("{:?}", token.kind)));
            }
        },

        None => return Err(ParserError::UnexpectedToken("end of input".to_string())),
    };

    match tokens.next() {
        Some(token) if token.kind == TokenKind::In => {}
        Some(token) => {
            return Err(ParserError::UnexpectedToken(format!("{:?}", token.kind)));
        }
        None => return Err(ParserError::UnexpectedToken("end of input".to_string())),
    }

    let iterable = parse_expression_bp(tokens, 0)?;

    skip_newlines(tokens);

    let body = parse_block(tokens)?;

    Ok(Statement::For {
        variable,
        iterable,
        body,
    })
}

/// Parse a match expression.
///
/// Examples:
///
/// match score {
///     90 => "A",
///     80 => "B",
///     _ => "F"
/// }
///
/// match value {
///     1 | 2 | 3 => "Small",
///     4..10 => "Medium",
///     10..=20 if value >= 15 => "Large",
///     _ => "Other"
/// }
fn parse_match_expression(tokens: &mut TokenStream) -> ParseResult<Expression> {
    // Parse the expression being matched.
    let value = parse_expression_bp(tokens, 0)?;

    skip_newlines(tokens);

    // Require opening brace.
    expect_symbol(tokens, '{')?;

    let mut arms = Vec::new();

    loop {
        skip_newlines(tokens);

        match tokens.peek().map(|token| &token.kind) {
            Some(TokenKind::Symbol('}')) => {
                tokens.next();
                break;
            }

            Some(TokenKind::Eof) | None => {
                return Err(ParserError::UnexpectedToken(
                    "Unterminated match expression: expected '}'".into(),
                ));
            }

            _ => {}
        }

        // Parse the pattern, including any OR alternatives.
        let pattern = parse_match_pattern(tokens)?;

        // Optional guard: pattern if condition
        let guard = if matches!(tokens.peek().map(|token| &token.kind), Some(TokenKind::If)) {
            tokens.next();

            Some(parse_expression_bp(tokens, 0)?)
        } else {
            None
        };

        // Require the fat-arrow operator.
        match tokens.peek() {
            Some(token) if token.kind == TokenKind::FatArrow => {
                tokens.next();
            }

            other => {
                return Err(ParserError::UnexpectedToken(format!(
                    "Expected '=>' after match pattern and optional guard, found {:?}",
                    other.map(|token| &token.kind)
                )));
            }
        }

        // Parse the expression produced by this arm.
        let expression = parse_expression_bp(tokens, 0)?;

        arms.push(MatchArm {
            pattern,
            guard,
            expression,
        });

        // Consume an optional comma.
        if matches!(
            tokens.peek().map(|token| &token.kind),
            Some(TokenKind::Symbol(','))
        ) {
            tokens.next();
        } else if !matches!(
            tokens.peek().map(|token| &token.kind),
            Some(TokenKind::Newline) | Some(TokenKind::Symbol('}'))
        ) {
            return Err(ParserError::UnexpectedToken(format!(
                "Expected ',', newline, or '}}' after match arm, found {:?}",
                tokens.peek().map(|token| &token.kind)
            )));
        }
    }

    if arms.is_empty() {
        return Err(ParserError::UnexpectedToken(
            "Match expression requires at least one arm".to_string(),
        ));
    }

    Ok(Expression::Match {
        value: Box::new(value),
        arms,
    })
}

/// Parse a complete match pattern.
///
/// Supports:
/// - Literal patterns
/// - Wildcard patterns
/// - Numeric ranges
/// - OR patterns
fn parse_match_pattern(tokens: &mut TokenStream) -> ParseResult<MatchPattern> {
    let mut alternatives = Vec::new();

    alternatives.push(parse_match_pattern_atom(tokens)?);

    // A single | separates OR-pattern alternatives.
    // Logical || is a separate Operator token.
    while matches!(
        tokens.peek().map(|token| &token.kind),
        Some(TokenKind::Pipe)
    ) {
        tokens.next();

        alternatives.push(parse_match_pattern_atom(tokens)?);
    }

    if alternatives.len() == 1 {
        Ok(alternatives.remove(0))
    } else {
        Ok(MatchPattern::Or(alternatives))
    }
}

/// Parse an individual match pattern or range.
fn parse_match_pattern_atom(tokens: &mut TokenStream) -> ParseResult<MatchPattern> {
    let token = tokens.next().ok_or_else(|| {
        ParserError::UnexpectedToken("Expected a match pattern, found end of input".into())
    })?;

    let pattern = match token.kind {
        TokenKind::Integer(value) => MatchPattern::Int(value),

        TokenKind::Float(value) => MatchPattern::Float(value),

        TokenKind::StringLiteral(value) => MatchPattern::String(value),

        TokenKind::CharacterLiteral(value) => MatchPattern::Char(value),

        TokenKind::True => MatchPattern::Bool(true),

        TokenKind::False => MatchPattern::Bool(false),

        TokenKind::Identifier(name) if name == "_" => MatchPattern::Wildcard,

        other => {
            return Err(ParserError::UnexpectedToken(format!(
                "Invalid match pattern: {:?}",
                other
            )));
        }
    };

    // Check whether this pattern starts a numeric range.
    let inclusive = match tokens.peek().map(|token| &token.kind) {
        Some(TokenKind::Range) => {
            tokens.next();
            Some(false)
        }

        Some(TokenKind::RangeInclusive) => {
            tokens.next();
            Some(true)
        }

        _ => None,
    };

    let Some(inclusive) = inclusive else {
        return Ok(pattern);
    };

    // Only numeric literals can define range boundaries.
    if !matches!(&pattern, MatchPattern::Int(_) | MatchPattern::Float(_)) {
        return Err(ParserError::UnexpectedToken(
            "Range patterns require numeric boundaries".into(),
        ));
    }

    let end_token = tokens.next().ok_or_else(|| {
        ParserError::UnexpectedToken("Expected range endpoint, found end of input".into())
    })?;

    let end = match end_token.kind {
        TokenKind::Integer(value) => MatchPattern::Int(value),

        TokenKind::Float(value) => MatchPattern::Float(value),

        other => {
            return Err(ParserError::UnexpectedToken(format!(
                "Expected numeric range endpoint, found {:?}",
                other
            )));
        }
    };

    Ok(MatchPattern::Range {
        start: Box::new(pattern),
        end: Box::new(end),
        inclusive,
    })
}

/// Parse an array literal.
///
/// Examples:
/// [1, 2, 3]
/// ["Ada", "Grace", "Alan"]
/// [true, false]
fn parse_array_expression(tokens: &mut TokenStream) -> ParseResult<Expression> {
    let mut elements = Vec::new();

    skip_newlines(tokens);

    // Empty arrays are syntactically valid.
    // Semantic analysis will reject them until type annotations
    // for empty arrays are supported.
    if let Some(token) = tokens.peek() {
        if token.kind == TokenKind::RightBracket {
            tokens.next();

            return Ok(Expression::Array(elements));
        }
    }

    loop {
        let element = parse_expression_bp(tokens, 0)?;
        elements.push(element);

        skip_newlines(tokens);

        match tokens.next() {
            Some(token) if token.kind == TokenKind::Symbol(',') => {
                skip_newlines(tokens);

                if let Some(next) = tokens.peek() {
                    if next.kind == TokenKind::RightBracket {
                        tokens.next();
                        break;
                    }
                }
            }

            Some(token) if token.kind == TokenKind::RightBracket => {
                break;
            }

            Some(token) => {
                return Err(ParserError::UnexpectedToken(format!("{:?}", token.kind)));
            }

            None => {
                return Err(ParserError::UnexpectedToken("end of input".to_string()));
            }
        }
    }

    Ok(Expression::Array(elements))
}

/// Parse:
/// let name = expression
/// let name: Type = expression
fn parse_let_statement(tokens: &mut TokenStream) -> ParseResult<Statement> {
    let name = match tokens.next() {
        Some(Token {
            kind: TokenKind::Identifier(name),
            ..
        }) => name,

        _ => {
            return Err(ParserError::UnexpectedToken(
                "Expected identifier after 'let'".into(),
            ));
        }
    };

    let type_annotation = match tokens.peek().map(|token| &token.kind) {
        Some(TokenKind::Symbol(':')) => {
            tokens.next();

            let type_name = match tokens.next() {
                Some(Token {
                    kind: TokenKind::Identifier(name),
                    ..
                }) => name,

                _ => {
                    return Err(ParserError::UnexpectedToken(
                        "Expected type name after ':'".into(),
                    ));
                }
            };

            Some(parse_type_annotation(&type_name)?)
        }

        _ => None,
    };

    match tokens.next() {
        Some(Token {
            kind: TokenKind::Operator(operator),
            ..
        }) if operator == "=" => {}

        _ => {
            return Err(ParserError::UnexpectedToken(
                "Expected '=' in variable declaration".into(),
            ));
        }
    }

    let initializer = parse_expression_bp(tokens, 0)?;

    Ok(Statement::Let {
        name,
        type_annotation,
        initializer,
    })
}

/// Convert a type name into its AST representation.
fn parse_type_annotation(name: &str) -> ParseResult<TypeAnnotation> {
    match name {
        "Int" => Ok(TypeAnnotation::Int),
        "Float" => Ok(TypeAnnotation::Float),
        "Bool" => Ok(TypeAnnotation::Bool),
        "Char" => Ok(TypeAnnotation::Char),
        "String" => Ok(TypeAnnotation::String),
        "Null" => Ok(TypeAnnotation::Null),

        _ => Err(ParserError::UnexpectedToken(format!(
            "Unknown type annotation: {}",
            name
        ))),
    }
}

/// Pratt-style expression parser.
fn parse_expression_bp(tokens: &mut TokenStream, min_precedence: u8) -> ParseResult<Expression> {
    let left = parse_prefix(tokens)?;

    parse_expression_bp_from_left(left, tokens, min_precedence)
}

/// Continue parsing an expression from an existing left-hand expression.
fn parse_expression_bp_from_left(
    mut left: Expression,
    tokens: &mut TokenStream,
    min_precedence: u8,
) -> ParseResult<Expression> {
    loop {
        // Function calls have higher precedence than binary operators.
        if matches!(
            tokens.peek().map(|token| &token.kind),
            Some(TokenKind::Symbol('('))
        ) {
            if let Expression::Identifier(name) = left {
                left = parse_function_call(name, tokens)?;
                continue;
            } else {
                return Err(ParserError::UnexpectedToken(
                    "Only named functions can be called".into(),
                ));
            }
        }

        let operator = match tokens.peek() {
            Some(Token {
                kind: TokenKind::Operator(operator),
                ..
            }) => operator.clone(),

            _ => break,
        };

        let Some((binary_operator, precedence)) = get_binary_operator(&operator) else {
            break;
        };

        if precedence < min_precedence {
            break;
        }

        tokens.next();

        let right = parse_expression_bp(tokens, precedence + 1)?;

        left = Expression::Binary {
            left: Box::new(left),
            operator: binary_operator,
            right: Box::new(right),
        };
    }

    Ok(left)
}

/// Parse a function call:
///
/// add(10, 20)
fn parse_function_call(name: String, tokens: &mut TokenStream) -> ParseResult<Expression> {
    expect_symbol(tokens, '(')?;

    let mut arguments = Vec::new();

    skip_newlines(tokens);

    if !matches!(
        tokens.peek().map(|token| &token.kind),
        Some(TokenKind::Symbol(')'))
    ) {
        loop {
            let argument = parse_expression_bp(tokens, 0)?;
            arguments.push(argument);

            skip_newlines(tokens);

            match tokens.peek().map(|token| &token.kind) {
                Some(TokenKind::Symbol(',')) => {
                    tokens.next();
                    skip_newlines(tokens);
                }

                Some(TokenKind::Symbol(')')) => break,

                other => {
                    return Err(ParserError::UnexpectedToken(format!(
                        "Expected ',' or ')' after function argument, found {:?}",
                        other
                    )));
                }
            }
        }
    }

    expect_symbol(tokens, ')')?;

    Ok(Expression::Call { name, arguments })
}

/// Convert lexer interpolation segments into AST string segments.
fn parse_interpolated_string(parts: Vec<StringPart>) -> ParseResult<Expression> {
    let mut segments = Vec::with_capacity(parts.len());

    for part in parts {
        match part {
            StringPart::Text(text) => {
                segments.push(StringSegment::Text(text));
            }

            StringPart::Expression(source) => {
                let tokens = Lexer::tokenize(&source).map_err(|error| {
                    ParserError::UnexpectedToken(format!(
                        "Invalid interpolation expression '{}': {}",
                        source, error
                    ))
                })?;

                let expression = parse_expression(tokens)?;

                segments.push(StringSegment::Expression(expression));
            }
        }
    }

    Ok(Expression::InterpolatedString(segments))
}

/// Parse unary expressions and primary expressions.
fn parse_prefix(tokens: &mut TokenStream) -> ParseResult<Expression> {
    let token = tokens.next().ok_or_else(|| {
        ParserError::UnexpectedToken("Expected expression, found end of input".into())
    })?;

    match token.kind {
        TokenKind::Integer(value) => Ok(Expression::IntegerLiteral(value)),

        TokenKind::Float(value) => Ok(Expression::FloatLiteral(value)),

        TokenKind::StringLiteral(value) => Ok(Expression::StringLiteral(value)),

        TokenKind::CharacterLiteral(value) => Ok(Expression::CharacterLiteral(value)),

        TokenKind::InterpolatedString(parts) => parse_interpolated_string(parts),

        TokenKind::True => Ok(Expression::BooleanLiteral(true)),

        TokenKind::False => Ok(Expression::BooleanLiteral(false)),

        TokenKind::Null => Ok(Expression::NullLiteral),

        TokenKind::Identifier(name) => Ok(Expression::Identifier(name)),

        TokenKind::Operator(operator) if operator == "-" => {
            let operand = parse_expression_bp(tokens, 7)?;

            Ok(Expression::Unary {
                operator: UnaryOperator::Negate,
                operand: Box::new(operand),
            })
        }

        TokenKind::Operator(operator) if operator == "!" => {
            let operand = parse_expression_bp(tokens, 7)?;

            Ok(Expression::Unary {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
            })
        }

        TokenKind::Symbol('(') => {
            let expression = parse_expression_bp(tokens, 0)?;

            expect_symbol(tokens, ')')?;

            Ok(expression)
        }

        TokenKind::LeftBracket => parse_array_expression(tokens),
        TokenKind::Match => parse_match_expression(tokens),

        other => Err(ParserError::UnexpectedToken(format!(
            "Expected expression, found {:?}",
            other
        ))),
    }
}

/// Convert an operator into its AST representation and precedence.
fn get_binary_operator(operator: &str) -> Option<(BinaryOperator, u8)> {
    match operator {
        "||" => Some((BinaryOperator::Or, 1)),
        "&&" => Some((BinaryOperator::And, 2)),

        "==" => Some((BinaryOperator::Equal, 3)),
        "!=" => Some((BinaryOperator::NotEqual, 3)),

        "<" => Some((BinaryOperator::LessThan, 4)),
        ">" => Some((BinaryOperator::GreaterThan, 4)),
        "<=" => Some((BinaryOperator::LessThanOrEqual, 4)),
        ">=" => Some((BinaryOperator::GreaterThanOrEqual, 4)),

        "+" => Some((BinaryOperator::Add, 5)),
        "-" => Some((BinaryOperator::Subtract, 5)),

        "*" => Some((BinaryOperator::Multiply, 6)),
        "/" => Some((BinaryOperator::Divide, 6)),
        "%" => Some((BinaryOperator::Modulo, 6)),

        _ => None,
    }
}

/// Require a specific punctuation symbol.
fn expect_symbol(tokens: &mut TokenStream, expected: char) -> ParseResult<()> {
    match tokens.next() {
        Some(Token {
            kind: TokenKind::Symbol(actual),
            ..
        }) if actual == expected => Ok(()),

        other => Err(ParserError::UnexpectedToken(format!(
            "Expected '{}', found {:?}",
            expected,
            other.map(|token| token.kind)
        ))),
    }
}

/// Consume newlines only.
fn skip_newlines(tokens: &mut TokenStream) {
    while matches!(
        tokens.peek().map(|token| &token.kind),
        Some(TokenKind::Newline)
    ) {
        tokens.next();
    }
}

/// Consume statement separators.
fn skip_statement_separators(tokens: &mut TokenStream) {
    loop {
        match tokens.peek().map(|token| &token.kind) {
            Some(TokenKind::Newline) | Some(TokenKind::Symbol(';')) => {
                tokens.next();
            }

            _ => break,
        }
    }
}
