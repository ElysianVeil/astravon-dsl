use astravon_dsl::ast::nodes::Program;
use astravon_dsl::bytecode::{decoder::decode, encoder::encode};
use astravon_dsl::codegen::avm_backend::compile_program;
use astravon_dsl::lexer::tokenizer::Lexer;
use astravon_dsl::parser::grammar::parse_program;
use astravon_dsl::runtime::{AVM, executor};
use astravon_dsl::semantics::analyzer::Analyzer;

fn run_program(source: &str) -> Result<String, String> {
    let tokens = Lexer::tokenize(source)?;

    let program: Program = parse_program(tokens).map_err(|error| format!("{:?}", error))?;

    let mut analyzer = Analyzer::new();

    analyzer
        .analyze_program(&program)
        .map_err(|error| format!("{:?}", error))?;

    let instructions = compile_program(&program)?;

    let bytecode = encode(&instructions)?;

    let decoded = decode(&bytecode)?;

    let mut vm = AVM::new(decoded);

    executor::execute_with_output(&mut vm)
}

#[test]
fn test_match_or_pattern() {
    let source = r#"
        let number = 2;

        let result = match number {
            1 | 2 | 3 => "Small",
            _ => "Other"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Small\n");
}

#[test]
fn test_match_exclusive_range() {
    let source = r#"
        let number = 7;

        let result = match number {
            1..5 => "Low",
            5..10 => "Medium",
            _ => "High"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Medium\n");
}

#[test]
fn test_match_inclusive_range() {
    let source = r#"
        let number = 10;

        let result = match number {
            1..=10 => "Included",
            _ => "Excluded"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Included\n");
}

#[test]
fn test_match_guard_true() {
    let source = r#"
        let number = 18;

        let result = match number {
            10..=20 if number >= 15 => "Large",
            _ => "Other"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Large\n");
}

#[test]
fn test_match_guard_false_continues_to_next_arm() {
    let source = r#"
        let number = 12;

        let result = match number {
            10..=20 if number >= 15 => "Large",
            10..=20 => "Medium",
            _ => "Other"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Medium\n");
}

#[test]
fn test_match_or_pattern_with_guard() {
    let source = r#"
        let number = 3;

        let result = match number {
            1 | 2 | 3 if number == 3 => "Matched",
            _ => "Other"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Matched\n");
}

#[test]
fn test_match_float_range() {
    let source = r#"
        let number = 3.5;

        let result = match number {
            1.0..5.0 => "Within range",
            _ => "Outside range"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Within range\n");
}

#[test]
fn test_match_wildcard_after_failed_guard() {
    let source = r#"
        let number = 5;

        let result = match number {
            1..=10 if number > 10 => "First",
            _ => "Fallback"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Fallback\n");
}
