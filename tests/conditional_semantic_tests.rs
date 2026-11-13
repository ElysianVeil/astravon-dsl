use astravon_dsl::lexer::tokenizer::Lexer;
use astravon_dsl::parser::grammar::parse_program;
use astravon_dsl::semantics::analyzer::{Analyzer, SemanticError};

fn analyze(source: &str) -> Result<(), SemanticError> {
    let tokens = Lexer::tokenize(source)
        .map_err(|_| SemanticError::UnsupportedExpression("Lexical analysis failed".into()))?;

    let program = parse_program(tokens)
        .map_err(|_| SemanticError::UnsupportedExpression("Parsing failed".into()))?;

    Analyzer::new().analyze_program(&program)
}

#[test]
fn test_if_with_boolean_condition() {
    let source = r#"
        let age = 20;

        if age >= 18 {
            print "Adult";
        }
    "#;

    assert!(analyze(source).is_ok());
}

#[test]
fn test_if_else_with_boolean_condition() {
    let source = r#"
        let age = 16;

        if age >= 18 {
            print "Adult";
        } else {
            print "Minor";
        }
    "#;

    assert!(analyze(source).is_ok());
}

#[test]
fn test_reject_integer_condition() {
    let source = r#"
        let age = 20;

        if age {
            print "Adult";
        }
    "#;

    assert!(matches!(
        analyze(source),
        Err(SemanticError::TypeMismatch {
            expected: astravon_dsl::semantics::analyzer::Type::Bool,
            ..
        })
    ));
}

#[test]
fn test_reject_string_condition() {
    let source = r#"
        if "Astravon" {
            print "Hello";
        }
    "#;

    assert!(matches!(
        analyze(source),
        Err(SemanticError::TypeMismatch {
            expected: astravon_dsl::semantics::analyzer::Type::Bool,
            ..
        })
    ));
}

#[test]
fn test_logical_condition() {
    let source = r#"
        let age = 20;
        let active = true;

        if age >= 18 && active {
            print "Eligible";
        }
    "#;

    assert!(analyze(source).is_ok());
}

#[test]
fn test_nested_conditionals() {
    let source = r#"
        let age = 25;
        let active = true;

        if age >= 18 {
            if active {
                print "Active adult";
            } else {
                print "Inactive adult";
            }
        } else {
            print "Minor";
        }
    "#;

    assert!(analyze(source).is_ok());
}

#[test]
fn test_branch_variables_do_not_leak() {
    let source = r#"
        let active = true;

        if active {
            let message = "Inside branch";
            print message;
        }

        print message;
    "#;

    assert!(matches!(
        analyze(source),
        Err(SemanticError::UndeclaredVariable(name))
            if name == "message"
    ));
}

#[test]
fn test_else_branch_variables_do_not_leak() {
    let source = r#"
        let active = false;

        if active {
            print "Active";
        } else {
            let message = "Inside else";
            print message;
        }

        print message;
    "#;

    assert!(matches!(
        analyze(source),
        Err(SemanticError::UndeclaredVariable(name))
            if name == "message"
    ));
}

#[test]
fn test_branch_can_access_outer_variables() {
    let source = r#"
        let age = 20;

        if age >= 18 {
            let message = "Adult";
            print age;
            print message;
        }
    "#;

    assert!(analyze(source).is_ok());
}

#[test]
fn test_branch_assignment_to_outer_variable() {
    let source = r#"
        let age = 20;

        if age >= 18 {
            age = 21;
        }

        print age;
    "#;

    assert!(analyze(source).is_ok());
}
