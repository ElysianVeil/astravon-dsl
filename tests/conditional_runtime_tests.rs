use astravon_dsl::runtime::executor::execute_with_output;
use astravon_dsl::runtime::{AVM, OpCode, Value};

#[test]
fn test_jump_if_false_when_true() {
    let instructions = vec![
        OpCode::PushBool(true),
        OpCode::JumpIfFalse(4),
        OpCode::PushString("True branch".into()),
        OpCode::Print,
        OpCode::PushString("Finished".into()),
        OpCode::Print,
    ];

    let mut vm = AVM::new(instructions);

    let output = execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "True branch\nFinished\n");
}

#[test]
fn test_jump_if_false_when_false() {
    let instructions = vec![
        OpCode::PushBool(false),
        OpCode::JumpIfFalse(4),
        OpCode::PushString("True branch".into()),
        OpCode::Print,
        OpCode::PushString("False branch".into()),
        OpCode::Print,
    ];

    let mut vm = AVM::new(instructions);

    let output = execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "False branch\n");
}

#[test]
fn test_jump_if_false_rejects_non_boolean() {
    let instructions = vec![OpCode::PushInt(10), OpCode::JumpIfFalse(2)];

    let mut vm = AVM::new(instructions);

    let result = execute_with_output(&mut vm);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("requires a Boolean"));
}

#[test]
fn test_jump_if_false_rejects_stack_underflow() {
    let instructions = vec![OpCode::JumpIfFalse(1)];

    let mut vm = AVM::new(instructions);

    let result = execute_with_output(&mut vm);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Stack underflow"));
}

#[test]
fn test_jump_if_false_rejects_invalid_target() {
    let instructions = vec![OpCode::PushBool(false), OpCode::JumpIfFalse(10)];

    let mut vm = AVM::new(instructions);

    let result = execute_with_output(&mut vm);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Invalid conditional jump target")
    );
}

#[test]
fn test_jump_if_false_consumes_condition() {
    let instructions = vec![
        OpCode::PushBool(true),
        OpCode::JumpIfFalse(3),
        OpCode::PushInt(42),
    ];

    let mut vm = AVM::new(instructions);

    execute_with_output(&mut vm).unwrap();

    assert_eq!(vm.stack, vec![Value::Int(42)]);
}
