use astravon_dsl::ast::nodes::{BinaryOperator, Expression, Statement};
use astravon_dsl::lexer::tokenizer::Lexer;
use astravon_dsl::parser::grammar::parse_program;

fn parse(
    source: &str,
) -> Result<astravon_dsl::ast::nodes::Program, astravon_dsl::parser::ParserError> {
    let tokens = Lexer::tokenize(source).unwrap();
    parse_program(tokens)
}

#[test]
fn test_parse_simple_while_loop() {
    let program = parse(
        r#"
        while true {
            print "Running";
        }
        "#,
    )
    .unwrap();

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::While { condition, body } => {
            assert_eq!(*condition, Expression::BooleanLiteral(true));

            assert_eq!(body.len(), 1);

            assert!(matches!(body[0], Statement::Print(_)));
        }

        other => panic!("Expected While statement, found {:?}", other),
    }
}

#[test]
fn test_parse_while_with_comparison() {
    let program = parse(
        r#"
        let count = 0;

        while count < 5 {
            count = count + 1;
        }
        "#,
    )
    .unwrap();

    match &program.statements[1] {
        Statement::While { condition, body } => {
            assert!(matches!(
                condition,
                Expression::Binary {
                    operator: BinaryOperator::LessThan,
                    ..
                }
            ));

            assert_eq!(body.len(), 1);

            assert!(matches!(body[0], Statement::Assignment { .. }));
        }

        other => panic!("Expected While statement, found {:?}", other),
    }
}

#[test]
fn test_parse_nested_while_loops() {
    let program = parse(
        r#"
        while true {
            while false {
                print "Nested";
            }
        }
        "#,
    )
    .unwrap();

    match &program.statements[0] {
        Statement::While { body, .. } => {
            assert_eq!(body.len(), 1);

            assert!(matches!(body[0], Statement::While { .. }));
        }

        other => panic!("Expected While statement, found {:?}", other),
    }
}

#[test]
fn test_parse_while_with_if_statement() {
    let program = parse(
        r#"
        while true {
            if true {
                print "Inside";
            }
        }
        "#,
    )
    .unwrap();

    match &program.statements[0] {
        Statement::While { body, .. } => {
            assert!(matches!(body[0], Statement::If { .. }));
        }

        other => panic!("Expected While statement, found {:?}", other),
    }
}

#[test]
fn test_parse_while_with_function_call() {
    let program = parse(
        r#"
        while true {
            calculate();
        }
        "#,
    )
    .unwrap();

    match &program.statements[0] {
        Statement::While { body, .. } => {
            assert!(matches!(
                body[0],
                Statement::Expression(Expression::Call { .. })
            ));
        }

        other => panic!("Expected While statement, found {:?}", other),
    }
}

#[test]
fn test_reject_unterminated_while_block() {
    let result = parse(
        r#"
        while true {
            print "Running";
        "#,
    );

    assert!(result.is_err());
}
