use astravon_dsl::ast::nodes::{Expression, StringSegment};
use astravon_dsl::lexer::tokenizer::{Lexer, TokenKind};
use astravon_dsl::parser::grammar;

#[test]
fn test_escaped_braces_are_literal_text() {
    let tokens = Lexer::tokenize(r#""Literal \{name\}""#).unwrap();

    assert_eq!(tokens.len(), 2);

    assert_eq!(
        tokens[0].kind,
        TokenKind::StringLiteral("Literal {name}".to_string())
    );
}

#[test]
fn test_interpolation_without_surrounding_text() {
    let tokens = Lexer::tokenize(r#""{name}""#).unwrap();

    match &tokens[0].kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 1);

            assert!(matches!(
                &parts[0],
                astravon_dsl::lexer::tokenizer::StringPart::Expression(source)
                    if source == "name"
            ));
        }
        other => panic!("Expected interpolated string, got {:?}", other),
    }
}

#[test]
fn test_consecutive_interpolations() {
    let tokens = Lexer::tokenize(r#""{first}{second}""#).unwrap();

    match &tokens[0].kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 2);
        }
        other => panic!("Expected interpolated string, got {:?}", other),
    }
}

#[test]
fn test_nested_interpolated_string() {
    let source = r#""Outer {"Inner {name}"}""#;

    let tokens = Lexer::tokenize(source).unwrap();
    let expression = grammar::parse_expression(tokens).unwrap();

    match expression {
        Expression::InterpolatedString(outer_segments) => {
            assert_eq!(outer_segments.len(), 2);

            assert!(matches!(
                &outer_segments[0],
                StringSegment::Text(text) if text == "Outer "
            ));

            match &outer_segments[1] {
                StringSegment::Expression(Expression::InterpolatedString(inner_segments)) => {
                    assert_eq!(inner_segments.len(), 2);

                    assert!(matches!(
                        &inner_segments[0],
                        StringSegment::Text(text) if text == "Inner "
                    ));

                    assert!(matches!(
                        &inner_segments[1],
                        StringSegment::Expression(
                            Expression::Identifier(name)
                        ) if name == "name"
                    ));
                }

                other => panic!("Expected nested interpolated string, got {:?}", other),
            }
        }

        other => panic!("Expected interpolated string, got {:?}", other),
    }
}

#[test]
fn test_empty_interpolation_is_rejected() {
    let result = Lexer::tokenize(r#""Value: {}""#);

    assert!(result.is_err());
}

#[test]
fn test_unterminated_interpolation_is_rejected() {
    let result = Lexer::tokenize(r#""Value: {name""#);

    assert!(result.is_err());
}

#[test]
fn test_invalid_interpolation_expression_is_rejected() {
    let tokens = Lexer::tokenize(r#""Value: {age +}""#).unwrap();

    let result = grammar::parse_expression(tokens);

    assert!(result.is_err());
}
