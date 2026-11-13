use astravon_dsl::{
    bytecode, compile_source,
    runtime::{self, AVM},
};

/// Compile, decode, and execute an Astravon program.
fn run_program(source: &str) -> Result<String, String> {
    // Compile source into AVM bytecode.
    let bytecode = compile_source(source)?;

    // Decode bytecode into AVM instructions.
    let instructions = bytecode::decoder::decode(&bytecode)?;

    // Initialize the virtual machine.
    let mut vm = AVM::new(instructions);

    // Execute and capture output.
    runtime::executor::execute_with_output(&mut vm)
}

// -------------------------------------------------------------
// Basic function execution
// -------------------------------------------------------------

#[test]
fn test_function_return_value() {
    let source = r#"
        fn get_answer() -> Int {
            return 42;
        }

        print get_answer();
    "#;

    let output = run_program(source).unwrap();

    assert_eq!(output, "42\n");
}

#[test]
fn test_function_with_multiple_parameters() {
    let source = r#"
        fn add(a: Int, b: Int) -> Int {
            return a + b;
        }

        print add(20, 22);
    "#;

    let output = run_program(source).unwrap();

    assert_eq!(output, "42\n");
}

// -------------------------------------------------------------
// Nested and forward function calls
// -------------------------------------------------------------

#[test]
fn test_nested_function_calls() {
    let source = r#"
        fn multiply(a: Int, b: Int) -> Int {
            return a * b;
        }

        fn add(a: Int, b: Int) -> Int {
            return a + b;
        }

        print add(10, multiply(2, 5));
    "#;

    let output = run_program(source).unwrap();

    assert_eq!(output, "20\n");
}

#[test]
fn test_forward_function_call() {
    let source = r#"
        print double(21);

        fn double(value: Int) -> Int {
            return value * 2;
        }
    "#;

    let output = run_program(source).unwrap();

    assert_eq!(output, "42\n");
}

// -------------------------------------------------------------
// Function scope
// -------------------------------------------------------------

#[test]
fn test_local_variables_are_isolated_between_calls() {
    let source = r#"
        fn calculate(value: Int) -> Int {
            let result: Int = value + 1;
            return result;
        }

        let first = calculate(10);
        let second = calculate(20);

        print first;
        print second;
    "#;

    let output = run_program(source).unwrap();

    assert_eq!(output, "11\n21\n");
}

#[test]
fn test_function_can_access_global_variable() {
    let source = r#"
        let base: Int = 10;

        fn add_base(value: Int) -> Int {
            return base + value;
        }

        print add_base(32);
    "#;

    let output = run_program(source).unwrap();

    assert_eq!(output, "42\n");
}

// -------------------------------------------------------------
// Void functions
// -------------------------------------------------------------

#[test]
fn test_void_function() {
    let source = r#"
        fn greet(name: String) {
            print "Hello {name}!";
        }

        greet("Astravon");
    "#;

    let output = run_program(source).unwrap();

    assert_eq!(output, "Hello Astravon!\n");
}

// -------------------------------------------------------------
// Functions and string interpolation
// -------------------------------------------------------------

#[test]
fn test_function_call_inside_string_interpolation() {
    let source = r#"
        fn add(a: Int, b: Int) -> Int {
            return a + b;
        }

        print "Calculation: {add(20, 22)}";
    "#;

    let output = run_program(source).unwrap();

    assert_eq!(output, "Calculation: 42\n");
}

// -------------------------------------------------------------
// Runtime safety
// -------------------------------------------------------------

#[test]
fn test_recursive_function_respects_maximum_call_depth() {
    let source = r#"
        fn recurse() {
            recurse();
        }

        recurse();
    "#;

    let result = run_program(source);

    assert!(result.is_err());

    let error = result.unwrap_err();

    assert!(
        error.contains("Maximum function call depth"),
        "Unexpected error: {}",
        error
    );
}
