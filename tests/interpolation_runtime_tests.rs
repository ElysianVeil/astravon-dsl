use astravon_dsl::runtime::{AVM, OpCode, Value, executor};

#[test]
fn test_build_string_with_multiple_values() {
    let opcodes = vec![
        OpCode::PushString("Hello ".to_string()),
        OpCode::PushString("Astravon".to_string()),
        OpCode::PushString("!".to_string()),
        OpCode::BuildString(3),
        OpCode::Print,
    ];

    let mut vm = AVM::new(opcodes);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "Hello Astravon!\n");
}

#[test]
fn test_build_string_with_different_value_types() {
    let opcodes = vec![
        OpCode::PushString("Age: ".to_string()),
        OpCode::PushInt(20),
        OpCode::PushString(", active: ".to_string()),
        OpCode::PushBool(true),
        OpCode::PushString(", initial: ".to_string()),
        OpCode::PushChar('A'),
        OpCode::PushString(", value: ".to_string()),
        OpCode::PushFloat(1.68),
        OpCode::PushString(", missing: ".to_string()),
        OpCode::PushNull,
        OpCode::BuildString(10),
        OpCode::Print,
    ];

    let mut vm = AVM::new(opcodes);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(
        output,
        "Age: 20, active: true, initial: A, value: 1.68, missing: null\n"
    );
}

#[test]
fn test_build_empty_string() {
    let opcodes = vec![OpCode::BuildString(0), OpCode::Print];

    let mut vm = AVM::new(opcodes);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "\n");
}

#[test]
fn test_build_string_preserves_stack_order() {
    let opcodes = vec![
        OpCode::PushString("First".to_string()),
        OpCode::PushString("Second".to_string()),
        OpCode::PushString("Third".to_string()),
        OpCode::BuildString(3),
    ];

    let mut vm = AVM::new(opcodes);

    executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(vm.stack, vec![Value::Str("FirstSecondThird".to_string())]);
}

#[test]
fn test_build_string_stack_underflow() {
    let opcodes = vec![
        OpCode::PushString("Only one".to_string()),
        OpCode::BuildString(2),
    ];

    let mut vm = AVM::new(opcodes);

    let result = executor::execute_with_output(&mut vm);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Stack underflow"));
}
