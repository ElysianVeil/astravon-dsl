use astravon_dsl::{lexer::tokenizer, parser::grammar};

#[test]
fn test_simple_expression_parse() {
    let input = "5 + 10";
    let tokens = tokenizer::tokenize(input).unwrap();
    let ast = grammar::parse(tokens).unwrap();
    assert!(ast.is_expression());
}
