use astravon_dsl::{
    bytecode::{decoder, encoder},
    codegen::avm_backend,
    lexer::tokenizer::Lexer,
    parser::grammar,
    runtime::{AVM, executor},
    semantics::analyzer,
};

fn run_program(source: &str) -> Result<String, String> {
    // 1. Tokenize
    let tokens = Lexer::tokenize(source)?;

    // 2. Parse
    let program = grammar::parse_program(tokens).map_err(|error| format!("{error:?}"))?;

    // 3. Semantic analysis
    let mut analyzer = analyzer::Analyzer::new();
    analyzer
        .analyze_program(&program)
        .map_err(|error| format!("{error:?}"))?;

    // 4. Generate AVM instructions
    let instructions = avm_backend::compile_program(&program)?;

    // 5. Encode bytecode
    let bytecode = encoder::encode(&instructions)?;

    // 6. Decode bytecode
    let decoded_instructions = decoder::decode(&bytecode)?;

    // 7. Create VM and execute
    let mut vm = AVM::new(decoded_instructions);

    executor::execute_with_output(&mut vm)
}

#[test]
fn test_for_loop_break() {
    let code = r#"
        let numbers = [10, 20, 30, 40];

        for number in numbers {
            if number == 30 {
                break;
            }
            print number;
        }
    "#;

    let output = run_program(code).unwrap();

    assert_eq!(output, "10\n20\n");
}

#[test]
fn test_for_loop_continue() {
    let code = r#"
        let numbers = [1, 2, 3, 4, 5];

        for number in numbers {
            if number == 3 {
                continue;
            }
            print number;
        }
    "#;

    let output = run_program(code).unwrap();

    assert_eq!(output, "1\n2\n4\n5\n");
}
