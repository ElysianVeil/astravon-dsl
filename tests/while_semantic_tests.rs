use astravon_dsl::ast::nodes::Program;
use astravon_dsl::lexer::tokenizer::Lexer;
use astravon_dsl::parser::grammar::parse_program;
use astravon_dsl::semantics::analyzer::{Analyzer, SemanticError};

fn analyze(source: &str) -> Result<(), SemanticError> {
    let tokens = Lexer::tokenize(source).expect("Tokenization should succeed");

    let program: Program = parse_program(tokens).expect("Parsing should succeed");

    let mut analyzer = Analyzer::new();

    analyzer.analyze_program(&program)
}

#[test]
fn accepts_boolean_while_condition() {
    let source = r#"
        let count: Int = 0;

        while count < 5 {
            count = count + 1;
        }
    "#;

    assert!(analyze(source).is_ok());
}

#[test]
fn rejects_integer_while_condition() {
    let source = r#"
        while 10 {
            print "Running";
        }
    "#;

    assert!(matches!(
        analyze(source),
        Err(SemanticError::TypeMismatch { .. })
    ));
}

#[test]
fn rejects_string_while_condition() {
    let source = r#"
        while "Astravon" {
            print "Running";
        }
    "#;

    assert!(matches!(
        analyze(source),
        Err(SemanticError::TypeMismatch { .. })
    ));
}

#[test]
fn loop_body_can_access_outer_variables() {
    let source = r#"
        let count: Int = 0;

        while count < 3 {
            print count;
            count = count + 1;
        }
    "#;

    assert!(analyze(source).is_ok());
}

#[test]
fn loop_body_variables_do_not_leak() {
    let source = r#"
        while false {
            let temporary: Int = 10;
        }

        print temporary;
    "#;

    assert!(matches!(
        analyze(source),
        Err(SemanticError::UndeclaredVariable(name))
            if name == "temporary"
    ));
}

#[test]
fn nested_while_loops_are_valid() {
    let source = r#"
        let outer: Int = 0;

        while outer < 3 {
            let inner: Int = 0;

            while inner < 2 {
                inner = inner + 1;
            }

            outer = outer + 1;
        }
    "#;

    assert!(analyze(source).is_ok());
}

#[test]
fn loop_body_can_assign_outer_variables() {
    let source = r#"
        let total: Int = 0;

        while total < 10 {
            total = total + 2;
        }

        print total;
    "#;

    assert!(analyze(source).is_ok());
}
