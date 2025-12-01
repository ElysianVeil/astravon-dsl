use astravon_dsl::lexer::tokenizer; // assuming your crate is astravon_dsl

#[test]
fn test_identifier_tokenization() {
    let input = "let x = 5";
    let tokens = tokenizer::tokenize(input).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier("x".into()));
}
