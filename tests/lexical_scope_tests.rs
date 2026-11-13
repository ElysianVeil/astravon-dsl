use astravon_dsl::runtime::{AVM, OpCode, executor};

#[test]
fn test_global_scope_variable_is_removed_after_exit() {
    let mut vm = AVM::new(vec![
        OpCode::EnterScope,
        OpCode::PushString("temporary".to_string()),
        OpCode::DefineVar("message".to_string()),
        OpCode::ExitScope,
        OpCode::LoadVar("message".to_string()),
    ]);

    let result = executor::execute_with_output(&mut vm);

    assert!(result.unwrap_err().contains("Undefined variable"));
}

#[test]
fn test_variable_shadowing() {
    let mut vm = AVM::new(vec![
        OpCode::PushString("Global".to_string()),
        OpCode::DefineVar("message".to_string()),
        OpCode::EnterScope,
        OpCode::PushString("Local".to_string()),
        OpCode::DefineVar("message".to_string()),
        OpCode::LoadVar("message".to_string()),
        OpCode::Print,
        OpCode::ExitScope,
        OpCode::LoadVar("message".to_string()),
        OpCode::Print,
    ]);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "Local\nGlobal\n");
}

#[test]
fn test_branch_variable_does_not_prevent_global_redeclaration() {
    let mut vm = AVM::new(vec![
        OpCode::EnterScope,
        OpCode::PushString("First".to_string()),
        OpCode::DefineVar("message".to_string()),
        OpCode::ExitScope,
        OpCode::PushString("Second".to_string()),
        OpCode::DefineVar("message".to_string()),
        OpCode::LoadVar("message".to_string()),
        OpCode::Print,
    ]);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "Second\n");
}

#[test]
fn test_assignment_updates_nearest_scope() {
    let mut vm = AVM::new(vec![
        OpCode::PushInt(10),
        OpCode::DefineVar("count".to_string()),
        OpCode::EnterScope,
        OpCode::PushInt(20),
        OpCode::StoreVar("count".to_string()),
        OpCode::ExitScope,
        OpCode::LoadVar("count".to_string()),
        OpCode::Print,
    ]);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "20\n");
}

#[test]
fn test_nested_scope_restores_outer_variable() {
    let mut vm = AVM::new(vec![
        OpCode::PushString("Outer".to_string()),
        OpCode::DefineVar("message".to_string()),
        OpCode::EnterScope,
        OpCode::PushString("Inner".to_string()),
        OpCode::DefineVar("message".to_string()),
        OpCode::EnterScope,
        OpCode::PushString("Innermost".to_string()),
        OpCode::DefineVar("message".to_string()),
        OpCode::LoadVar("message".to_string()),
        OpCode::Print,
        OpCode::ExitScope,
        OpCode::LoadVar("message".to_string()),
        OpCode::Print,
        OpCode::ExitScope,
        OpCode::LoadVar("message".to_string()),
        OpCode::Print,
    ]);

    let output = executor::execute_with_output(&mut vm).unwrap();

    assert_eq!(output, "Innermost\nInner\nOuter\n");
}

#[test]
fn test_jump_if_false_stack_underflow_message() {
    let mut vm = AVM::new(vec![OpCode::JumpIfFalse(1)]);

    let error = executor::execute_with_output(&mut vm).unwrap_err();

    assert!(
        error.contains("Stack underflow"),
        "Unexpected error: {}",
        error
    );
}

#[test]
fn test_jump_if_false_rejects_non_boolean() {
    let mut vm = AVM::new(vec![OpCode::PushInt(10), OpCode::JumpIfFalse(2)]);

    let error = executor::execute_with_output(&mut vm).unwrap_err();

    assert!(
        error.contains("requires a Boolean"),
        "Unexpected error: {}",
        error
    );
}
