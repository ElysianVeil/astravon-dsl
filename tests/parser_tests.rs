use astravon_dsl::{
    ast::nodes::{BinaryOperator, Expression},
    lexer::tokenizer::Lexer,
    parser::grammar,
};

#[test]
fn test_simple_expression_parse() {
    let tokens = Lexer::tokenize("5 + 10").unwrap();

    let ast = grammar::parse_expression(tokens).unwrap();

    assert!(matches!(
        ast,
        Expression::Binary {
            operator: BinaryOperator::Add,
            ..
        }
    ));
}

#[test]
fn test_operator_precedence() {
    let tokens = Lexer::tokenize("5 + 10 * 2").unwrap();

    let ast = grammar::parse_expression(tokens).unwrap();

    match ast {
        Expression::Binary {
            operator: BinaryOperator::Add,
            right,
            ..
        } => {
            assert!(matches!(
                *right,
                Expression::Binary {
                    operator: BinaryOperator::Multiply,
                    ..
                }
            ));
        }

        _ => panic!("Expected addition at the root"),
    }
}

#[test]
fn test_parenthesized_expression() {
    let tokens = Lexer::tokenize("(5 + 10) * 2").unwrap();

    let ast = grammar::parse_expression(tokens).unwrap();

    assert!(matches!(
        ast,
        Expression::Binary {
            operator: BinaryOperator::Multiply,
            ..
        }
    ));
}
