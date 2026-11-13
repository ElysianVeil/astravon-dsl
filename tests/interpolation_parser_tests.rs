use astravon_dsl::ast::nodes::{Expression, StringSegment};
use astravon_dsl::lexer::tokenizer::Lexer;
use astravon_dsl::parser::grammar;

#[test]
fn test_parse_simple_interpolation() {
    let source = r#""Hello {name}!""#;
    let tokens = Lexer::tokenize(source).unwrap();
    let expression = grammar::parse_expression(tokens).unwrap();

    assert_eq!(
        expression,
        Expression::InterpolatedString(vec![
            StringSegment::Text("Hello ".to_string()),
            StringSegment::Expression(Expression::Identifier("name".to_string())),
            StringSegment::Text("!".to_string()),
        ])
    );
}

#[test]
fn test_parse_interpolation_with_arithmetic() {
    let source = r#""Result: {20 + 22}""#;
    let tokens = Lexer::tokenize(source).unwrap();
    let expression = grammar::parse_expression(tokens).unwrap();

    assert_eq!(
        expression,
        Expression::InterpolatedString(vec![
            StringSegment::Text("Result: ".to_string()),
            StringSegment::Expression(Expression::Binary {
                left: Box::new(Expression::IntegerLiteral(20)),
                operator: astravon_dsl::ast::nodes::BinaryOperator::Add,
                right: Box::new(Expression::IntegerLiteral(22)),
            }),
        ])
    );
}

#[test]
fn test_parse_multiple_interpolations() {
    let source = r#""{name} is {age} years old.""#;
    let tokens = Lexer::tokenize(source).unwrap();
    let expression = grammar::parse_expression(tokens).unwrap();

    assert_eq!(
        expression,
        Expression::InterpolatedString(vec![
            StringSegment::Expression(Expression::Identifier("name".to_string())),
            StringSegment::Text(" is ".to_string()),
            StringSegment::Expression(Expression::Identifier("age".to_string())),
            StringSegment::Text(" years old.".to_string()),
        ])
    );
}

#[test]
fn test_parse_interpolation_inside_print_statement() {
    let source = r#"
        let name = "Astravon";
        print "Welcome {name}!";
    "#;

    let tokens = Lexer::tokenize(source).unwrap();
    let program = grammar::parse_program(tokens).unwrap();

    assert_eq!(program.statements.len(), 2);

    match &program.statements[1] {
        astravon_dsl::ast::nodes::Statement::Print(Expression::InterpolatedString(_)) => {}
        other => panic!("Expected interpolated print statement, found {:?}", other),
    }
}

#[test]
fn test_parse_simple_interpolated_string() {
    let source = r#""Hello {name}!""#;

    let tokens = Lexer::tokenize(source).unwrap();
    let expression = grammar::parse_expression(tokens).unwrap();

    match expression {
        Expression::InterpolatedString(segments) => {
            assert_eq!(segments.len(), 3);

            assert_eq!(segments[0], StringSegment::Text("Hello ".to_string()));

            assert!(matches!(
                segments[1],
                StringSegment::Expression(Expression::Identifier(ref name))
                    if name == "name"
            ));

            assert_eq!(segments[2], StringSegment::Text("!".to_string()));
        }

        other => panic!("Expected interpolated string, got {:?}", other),
    }
}

#[test]
fn test_parse_arithmetic_interpolation() {
    let source = r#""Next year: {age + 1}""#;

    let tokens = Lexer::tokenize(source).unwrap();
    let expression = grammar::parse_expression(tokens).unwrap();

    match expression {
        Expression::InterpolatedString(segments) => {
            assert!(matches!(
                segments[1],
                StringSegment::Expression(Expression::Binary { .. })
            ));
        }

        other => panic!("Expected interpolated string, got {:?}", other),
    }
}

#[test]
fn test_parse_interpolation_with_parentheses() {
    let source = r#""Result: {(10 + 5) * 2}""#;

    let tokens = Lexer::tokenize(source).unwrap();
    let expression = grammar::parse_expression(tokens).unwrap();

    match expression {
        Expression::InterpolatedString(segments) => {
            assert!(matches!(
                segments[1],
                StringSegment::Expression(Expression::Binary { .. })
            ));
        }

        other => panic!("Expected interpolated string, got {:?}", other),
    }
}
