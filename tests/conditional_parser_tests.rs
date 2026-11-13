use astravon_dsl::ast::nodes::{Expression, Statement};
use astravon_dsl::lexer::tokenizer::Lexer;
use astravon_dsl::parser::grammar::parse_program;

fn parse(source: &str) -> Result<astravon_dsl::ast::nodes::Program, String> {
    let tokens = Lexer::tokenize(source)?;

    parse_program(tokens).map_err(|error| format!("{:?}", error))
}

#[test]
fn test_parse_if_statement() {
    let program = parse(
        r#"
        if true {
            print "Yes";
        }
        "#,
    )
    .unwrap();

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            assert_eq!(condition, &Expression::BooleanLiteral(true));

            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_none());
        }

        other => panic!("Expected If statement, found {:?}", other),
    }
}

#[test]
fn test_parse_if_else_statement() {
    let program = parse(
        r#"
        if true {
            print "Yes";
        } else {
            print "No";
        }
        "#,
    )
    .unwrap();

    match &program.statements[0] {
        Statement::If {
            then_branch,
            else_branch,
            ..
        } => {
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_some());
            assert_eq!(else_branch.as_ref().unwrap().len(), 1);
        }

        other => panic!("Expected If statement, found {:?}", other),
    }
}

#[test]
fn test_parse_else_if_chain() {
    let program = parse(
        r#"
        if score >= 90 {
            print "A";
        } else if score >= 80 {
            print "B";
        } else if score >= 70 {
            print "C";
        } else {
            print "D";
        }
        "#,
    )
    .unwrap();

    match &program.statements[0] {
        Statement::If {
            else_branch: Some(else_branch),
            ..
        } => {
            assert_eq!(else_branch.len(), 1);

            assert!(matches!(else_branch[0], Statement::If { .. }));
        }

        other => panic!("Expected conditional chain, found {:?}", other),
    }
}

#[test]
fn test_parse_nested_if_statement() {
    let program = parse(
        r#"
        if true {
            if false {
                print "A";
            } else {
                print "B";
            }
        }
        "#,
    )
    .unwrap();

    match &program.statements[0] {
        Statement::If { then_branch, .. } => {
            assert_eq!(then_branch.len(), 1);

            assert!(matches!(then_branch[0], Statement::If { .. }));
        }

        other => panic!("Expected nested If statement, found {:?}", other),
    }
}

#[test]
fn test_parse_if_with_logical_condition() {
    let program = parse(
        r#"
        if age >= 18 && has_id {
            print "Access granted";
        }
        "#,
    )
    .unwrap();

    match &program.statements[0] {
        Statement::If {
            condition: Expression::Binary { .. },
            ..
        } => {}

        other => panic!("Expected binary condition, found {:?}", other),
    }
}

#[test]
fn test_reject_else_without_if() {
    let result = parse(
        r#"
        else {
            print "Invalid";
        }
        "#,
    );

    assert!(result.is_err());
}
