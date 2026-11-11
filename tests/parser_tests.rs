use astravon_dsl::{lexer::tokenizer, parser::grammar};

#[test]
fn test_simple_expression_parse() {
    let input = "5 + 10";
    let tokens = tokenizer::Lexer::tokenize(input).unwrap();
    let ast = grammar::parse_expression(tokens).unwrap();
    assert!(ast.is_expression());
}
