use astravon_dsl::runtime::{AVM, OpCode, executor};

#[test]
fn test_variable_declaration_and_print() {
    let instructions = vec![
        OpCode::PushInt(20),
        OpCode::StoreVar("age".to_string()),
        OpCode::LoadVar("age".to_string()),
        OpCode::Print,
    ];

    let mut vm = AVM::new(instructions);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "20\n");
}

#[test]
fn test_variable_assignment() {
    let instructions = vec![
        OpCode::PushInt(20),
        OpCode::StoreVar("age".to_string()),
        OpCode::LoadVar("age".to_string()),
        OpCode::PushInt(1),
        OpCode::Add,
        OpCode::StoreVar("age".to_string()),
        OpCode::LoadVar("age".to_string()),
        OpCode::Print,
    ];

    let mut vm = AVM::new(instructions);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "21\n");
}

#[test]
fn test_comparison() {
    let instructions = vec![
        OpCode::PushInt(20),
        OpCode::PushInt(18),
        OpCode::GreaterThan,
        OpCode::Print,
    ];

    let mut vm = AVM::new(instructions);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "true\n");
}

#[test]
fn test_logical_operations() {
    let instructions = vec![
        OpCode::PushBool(true),
        OpCode::PushBool(false),
        OpCode::And,
        OpCode::Print,
    ];

    let mut vm = AVM::new(instructions);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "false\n");
}

#[test]
fn test_division_by_zero() {
    let instructions = vec![OpCode::PushInt(10), OpCode::PushInt(0), OpCode::Divide];

    let mut vm = AVM::new(instructions);

    assert!(executor::execute_with_output(&mut vm).is_err());
}

#[test]
fn test_undefined_variable() {
    let instructions = vec![OpCode::LoadVar("missing".to_string())];

    let mut vm = AVM::new(instructions);

    assert!(executor::execute_with_output(&mut vm).is_err());
}

#[test]
fn execution_stops_when_instruction_limit_is_exceeded() {
    let instructions = vec![
        OpCode::PushBool(true),
        OpCode::JumpIfFalse(3),
        OpCode::Jump(0),
    ];

    let mut vm = AVM::new(instructions);

    let result = executor::execute_with_output_and_limit(&mut vm, 100);

    assert!(matches!(
        result,
        Err(message) if message.contains("Maximum instruction limit")
    ));
}

#[test]
fn test_create_integer_array() {
    let mut vm = AVM::new(vec![
        OpCode::PushInt(10),
        OpCode::PushInt(20),
        OpCode::PushInt(30),
        OpCode::CreateArray(3),
        OpCode::Print,
    ]);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "[10, 20, 30]\n");
}

#[test]
fn test_array_length() {
    let mut vm = AVM::new(vec![
        OpCode::PushInt(10),
        OpCode::PushInt(20),
        OpCode::PushInt(30),
        OpCode::CreateArray(3),
        OpCode::ArrayLength,
        OpCode::Print,
    ]);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "3\n");
}

#[test]
fn test_array_get() {
    let mut vm = AVM::new(vec![
        OpCode::PushInt(10),
        OpCode::PushInt(20),
        OpCode::PushInt(30),
        OpCode::CreateArray(3),
        OpCode::PushInt(1),
        OpCode::ArrayGet,
        OpCode::Print,
    ]);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "20\n");
}
