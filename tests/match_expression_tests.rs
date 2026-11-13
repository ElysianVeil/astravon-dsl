use astravon_dsl::{
    bytecode::{decoder, encoder},
    codegen::avm_backend,
    lexer::tokenizer::Lexer,
    parser::grammar,
    runtime::{AVM, executor},
    semantics::analyzer,
};

fn run_program(source: &str) -> Result<String, String> {
    let tokens = Lexer::tokenize(source)?;

    let program = grammar::parse_program(tokens).map_err(|error| format!("{error:?}"))?;

    let mut analyzer = analyzer::Analyzer::new();

    analyzer
        .analyze_program(&program)
        .map_err(|error| format!("{error:?}"))?;

    let instructions = avm_backend::compile_program(&program)?;

    let bytecode = encoder::encode(&instructions)?;

    let decoded_instructions = decoder::decode(&bytecode)?;

    let mut vm = AVM::new(decoded_instructions);

    executor::execute_with_output(&mut vm)
}

#[test]
fn test_integer_match() {
    let source = r#"
        let score = 80;

        let grade = match score {
            90 => "A",
            80 => "B",
            70 => "C",
            _ => "F"
        };

        print grade;
    "#;

    assert_eq!(run_program(source).unwrap(), "B\n");
}

#[test]
fn test_boolean_match() {
    let source = r#"
        let enabled = true;

        let message = match enabled {
            true => "Active",
            false => "Inactive"
        };

        print message;
    "#;

    assert_eq!(run_program(source).unwrap(), "Active\n");
}

#[test]
fn test_string_match() {
    let source = r#"
        let command = "start";

        let result = match command {
            "start" => "Starting",
            "stop" => "Stopping",
            _ => "Unknown"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Starting\n");
}

#[test]
fn test_character_match() {
    let source = r#"
        let initial = 'A';

        let result = match initial {
            'A' => "Alpha",
            'B' => "Beta",
            _ => "Other"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Alpha\n");
}

#[test]
fn test_float_match() {
    let source = r#"
        let temperature = 37.5;

        let result = match temperature {
            36.5 => "Normal",
            37.5 => "Elevated",
            _ => "Other"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Elevated\n");
}

#[test]
fn test_match_expression_in_arithmetic() {
    let source = r#"
        let score = 80;

        let bonus = match score {
            80 => 20,
            _ => 0
        };

        print bonus + 10;
    "#;

    assert_eq!(run_program(source).unwrap(), "30\n");
}

#[test]
fn test_nested_match_expressions() {
    let source = r#"
        let category = 1;
        let level = 2;

        let result = match category {
            1 => match level {
                1 => "Beginner",
                2 => "Intermediate",
                _ => "Advanced"
            },
            _ => "Unknown"
        };

        print result;
    "#;

    assert_eq!(run_program(source).unwrap(), "Intermediate\n");
}

#[test]
fn test_match_inside_string_interpolation() {
    let source = r#"
        let score = 90;

        let message = "Grade: {match score {
            90 => "A",
            _ => "Other"
        }}";

        print message;
    "#;

    assert_eq!(run_program(source).unwrap(), "Grade: A\n");
}
