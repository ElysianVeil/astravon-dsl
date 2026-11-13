use astravon_dsl::{
    ast::nodes::{Expression, Program, Statement, TypeAnnotation},
    lexer::tokenizer::Lexer,
    parser::grammar,
};

#[test]
fn test_variable_declaration_and_print() {
    let source = r#"
        let age: Int = 20;
        print age;
    "#;

    let tokens = Lexer::tokenize(source).unwrap();
    let program = grammar::parse_program(tokens).unwrap();

    assert_eq!(program.statements.len(), 2);

    assert!(matches!(
        &program.statements[0],
        Statement::Let {
            name,
            type_annotation: Some(TypeAnnotation::Int),
            initializer: Expression::IntegerLiteral(20),
        } if name == "age"
    ));

    assert!(matches!(
        &program.statements[1],
        Statement::Print(Expression::Identifier(name))
            if name == "age"
    ));
}

#[test]
fn test_assignment_parsing() {
    let source = r#"
        let count = 0;
        count = count + 1;
    "#;

    let tokens = Lexer::tokenize(source).unwrap();
    let program = grammar::parse_program(tokens).unwrap();

    assert!(matches!(
        &program.statements[1],
        Statement::Assignment { name, .. }
            if name == "count"
    ));
}

#[test]
fn test_program_structure() {
    let tokens = Lexer::tokenize("let a = 20\nlet b = 22\nprint a + b").unwrap();

    let program: Program = grammar::parse_program(tokens).unwrap();

    assert_eq!(program.statements.len(), 3);
}
