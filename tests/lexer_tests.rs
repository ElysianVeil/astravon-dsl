use astravon_dsl::lexer::tokenizer::{Lexer, TokenKind};

#[test]
fn test_identifier_tokenization() {
    let tokens = Lexer::tokenize("let x = 5").unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("x".into()));
}

#[test]
fn test_arithmetic_operators() {
    let tokens = Lexer::tokenize("a + b - c * d / e % f").unwrap();

    let operators: Vec<String> = tokens
        .iter()
        .filter_map(|token| match &token.kind {
            TokenKind::Operator(operator) => Some(operator.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(operators, vec!["+", "-", "*", "/", "%"]);
}

#[test]
fn test_comparison_and_logical_operators() {
    let tokens = Lexer::tokenize("a == b && c != d || e >= f").unwrap();

    let operators: Vec<String> = tokens
        .iter()
        .filter_map(|token| match &token.kind {
            TokenKind::Operator(operator) => Some(operator.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(operators, vec!["==", "&&", "!=", "||", ">="]);
}

#[test]
fn test_literal_tokenization() {
    let tokens = Lexer::tokenize(r#"42 3.14 true false null 'A' "Astravon""#).unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Integer(42));
    assert_eq!(tokens[1].kind, TokenKind::Float(3.14));
    assert_eq!(tokens[2].kind, TokenKind::True);
    assert_eq!(tokens[3].kind, TokenKind::False);
    assert_eq!(tokens[4].kind, TokenKind::Null);
    assert_eq!(tokens[5].kind, TokenKind::CharacterLiteral('A'));
    assert_eq!(tokens[6].kind, TokenKind::StringLiteral("Astravon".into()));
}

#[test]
fn test_newline_preservation() {
    let tokens = Lexer::tokenize("let x = 5\nprint x").unwrap();

    assert!(tokens.iter().any(|token| token.kind == TokenKind::Newline));
}

#[test]
fn test_comment_handling() {
    let tokens = Lexer::tokenize("let x = 5 # comment\nprint x").unwrap();

    assert!(tokens.iter().any(|token| token.kind == TokenKind::Newline));

    assert!(!tokens.iter().any(|token| matches!(
        &token.kind,
        TokenKind::Identifier(value) if value == "comment"
    )));
}
