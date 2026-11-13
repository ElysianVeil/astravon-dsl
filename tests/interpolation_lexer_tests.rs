use astravon_dsl::lexer::tokenizer::{Lexer, StringPart, TokenKind};

#[test]
fn test_ordinary_string_remains_unchanged() {
    let tokens = Lexer::tokenize(r#""Hello Astravon!""#).unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::StringLiteral("Hello Astravon!".to_string())
    );
}

#[test]
fn test_simple_string_interpolation() {
    let tokens = Lexer::tokenize(r#""Hello {name}!""#).unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::InterpolatedString(vec![
            StringPart::Text("Hello ".to_string()),
            StringPart::Expression("name".to_string()),
            StringPart::Text("!".to_string()),
        ])
    );
}

#[test]
fn test_multiple_interpolations() {
    let tokens = Lexer::tokenize(r#""{name} is {age} years old.""#).unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::InterpolatedString(vec![
            StringPart::Expression("name".to_string()),
            StringPart::Text(" is ".to_string()),
            StringPart::Expression("age".to_string()),
            StringPart::Text(" years old.".to_string()),
        ])
    );
}

#[test]
fn test_escaped_braces_remain_literal() {
    let tokens = Lexer::tokenize(r#""Literal: \{name\}""#).unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::StringLiteral("Literal: {name}".to_string())
    );
}

#[test]
fn test_interpolation_with_arithmetic() {
    let tokens = Lexer::tokenize(r#""Result: {20 + 22}""#).unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::InterpolatedString(vec![
            StringPart::Text("Result: ".to_string()),
            StringPart::Expression("20 + 22".to_string()),
        ])
    );
}

#[test]
fn test_empty_interpolation_is_rejected() {
    assert!(Lexer::tokenize(r#""Hello {}""#).is_err());
}

#[test]
fn test_unterminated_interpolation_is_rejected() {
    assert!(Lexer::tokenize(r#""Hello {name""#).is_err());
}
