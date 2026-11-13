use astravon_dsl::ast::nodes::{Expression, Statement, TypeAnnotation};
use astravon_dsl::lexer::tokenizer::Lexer;
use astravon_dsl::parser::grammar;

#[test]
fn test_parse_function_declaration() {
    let source = r#"
        fn add(a: Int, b: Int) -> Int {
            return a + b;
        }
    "#;

    let tokens = Lexer::tokenize(source).unwrap();
    let program = grammar::parse_program(tokens).unwrap();

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Function {
            name,
            parameters,
            return_type,
            body,
        } => {
            assert_eq!(name, "add");
            assert_eq!(parameters.len(), 2);
            assert_eq!(parameters[0].name, "a");
            assert_eq!(parameters[0].type_annotation, TypeAnnotation::Int);
            assert_eq!(return_type, &Some(TypeAnnotation::Int));
            assert_eq!(body.len(), 1);

            assert!(matches!(
                &body[0],
                Statement::Return(Some(Expression::Binary { .. }))
            ));
        }

        other => panic!("Expected function declaration, got {:?}", other),
    }
}

#[test]
fn test_parse_function_call() {
    let source = "let result = add(20, 22);";

    let tokens = Lexer::tokenize(source).unwrap();
    let program = grammar::parse_program(tokens).unwrap();

    match &program.statements[0] {
        Statement::Let { initializer, .. } => {
            assert!(matches!(
                initializer,
                Expression::Call { name, arguments }
                    if name == "add" && arguments.len() == 2
            ));
        }

        other => panic!("Expected let statement, got {:?}", other),
    }
}

#[test]
fn test_parse_nested_function_calls() {
    let source = "let result = add(10, multiply(2, 5));";

    let tokens = Lexer::tokenize(source).unwrap();
    let program = grammar::parse_program(tokens).unwrap();

    match &program.statements[0] {
        Statement::Let { initializer, .. } => match initializer {
            Expression::Call { name, arguments } => {
                assert_eq!(name, "add");
                assert_eq!(arguments.len(), 2);

                assert!(matches!(
                    &arguments[1],
                    Expression::Call { name, .. }
                        if name == "multiply"
                ));
            }

            other => panic!("Expected function call, got {:?}", other),
        },

        other => panic!("Expected let statement, got {:?}", other),
    }
}

#[test]
fn test_parse_standalone_function_call() {
    let source = r#"
        fn greet(name: String) {
            print "Hello {name}!";
        }

        greet("Astravon");
    "#;

    let tokens = Lexer::tokenize(source).unwrap();
    let program = grammar::parse_program(tokens).unwrap();

    assert_eq!(program.statements.len(), 2);

    assert!(matches!(
        &program.statements[1],
        Statement::Expression(Expression::Call { name, arguments })
            if name == "greet" && arguments.len() == 1
    ));
}

#[test]
fn test_parse_return_without_value() {
    let source = r#"
        fn greet() {
            return;
        }
    "#;

    let tokens = Lexer::tokenize(source).unwrap();
    let program = grammar::parse_program(tokens).unwrap();

    match &program.statements[0] {
        Statement::Function { body, .. } => {
            assert!(matches!(&body[0], Statement::Return(None)));
        }

        other => panic!("Expected function declaration, got {:?}", other),
    }
}
