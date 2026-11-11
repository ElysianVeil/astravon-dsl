use astravon_dsl::lexer::tokenizer; // assuming your crate is astravon_dsl

#[test]
fn test_identifier_tokenization() {
    let input = "let x = 5";
    let tokens = tokenizer::Lexer::tokenize(input).unwrap();
    assert_eq!(tokens[0].kind, tokenizer::TokenKind::Let);
    assert_eq!(tokens[1].kind, tokenizer::TokenKind::Identifier("x".into()));
}
