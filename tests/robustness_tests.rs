use astravon_dsl::bytecode::{decoder, encoder};
use astravon_dsl::runtime::{AVM, OpCode, executor};

// ============================================
// Helper: Execute AVM instructions
// ============================================

fn run_program(opcodes: Vec<OpCode>) -> Result<String, String> {
    let mut vm = AVM::new(opcodes);
    executor::execute_with_output(&mut vm)
}

// ============================================
// 1. BYTECODE HEADER VALIDATION
// ============================================

#[test]
fn test_rejects_short_bytecode_header() {
    assert!(decoder::decode(b"AVM").is_err());
}

#[test]
fn test_rejects_invalid_magic_header() {
    assert!(decoder::decode(b"XYZ\x01").is_err());
}

#[test]
fn test_rejects_unsupported_bytecode_version() {
    assert!(decoder::decode(b"AVM\x03").is_err());
    assert!(decoder::decode(b"AVM\x05").is_err());
}

// ============================================
// 2. BYTECODE INSTRUCTION VALIDATION
// ============================================

#[test]
fn test_rejects_unknown_opcode() {
    assert!(decoder::decode(b"AVM\x01\xFF").is_err());
}

#[test]
fn test_rejects_truncated_integer() {
    let bytecode = b"AVM\x01\x01\x01\x02";

    assert!(decoder::decode(bytecode).is_err());
}

#[test]
fn test_rejects_truncated_float() {
    let bytecode = b"AVM\x01\x02\x01\x02";

    assert!(decoder::decode(bytecode).is_err());
}

#[test]
fn test_rejects_invalid_boolean() {
    let bytecode = b"AVM\x01\x03\x02";

    assert!(decoder::decode(bytecode).is_err());
}

#[test]
fn test_rejects_invalid_unicode_character() {
    // 0xD800 is not a valid Unicode scalar value.
    let bytecode = [b'A', b'V', b'M', 0x01, 0x04, 0x00, 0xD8, 0x00, 0x00];

    assert!(decoder::decode(&bytecode).is_err());
}

#[test]
fn test_rejects_truncated_string() {
    let bytecode = [
        b'A', b'V', b'M', 0x01, 0x06, 0x05, 0x00, 0x00, 0x00, b'A', b'B',
    ];

    assert!(decoder::decode(&bytecode).is_err());
}

#[test]
fn test_rejects_invalid_utf8_string() {
    let bytecode = [b'A', b'V', b'M', 0x01, 0x06, 0x01, 0x00, 0x00, 0x00, 0xFF];

    assert!(decoder::decode(&bytecode).is_err());
}

// ============================================
// 3. BYTECODE ROUND-TRIP VALIDATION
// ============================================

#[test]
fn test_all_opcodes_encode_and_decode() {
    let instructions = vec![
        OpCode::Nop,
        OpCode::PushInt(100),
        OpCode::PushFloat(25.5),
        OpCode::PushBool(true),
        OpCode::PushChar('A'),
        OpCode::PushNull,
        OpCode::PushString("Astravon".to_string()),
        OpCode::Print,
        OpCode::LoadVar("age".to_string()),
        OpCode::StoreVar("age".to_string()),
        OpCode::Add,
        OpCode::Subtract,
        OpCode::Multiply,
        OpCode::Divide,
        OpCode::Modulo,
        OpCode::Equal,
        OpCode::NotEqual,
        OpCode::LessThan,
        OpCode::GreaterThan,
        OpCode::LessThanOrEqual,
        OpCode::GreaterThanOrEqual,
        OpCode::And,
        OpCode::Or,
        OpCode::Negate,
        OpCode::Not,
    ];

    let encoded = encoder::encode(&instructions).unwrap();
    let decoded = decoder::decode(&encoded).unwrap();

    assert_eq!(instructions.len(), decoded.len());

    assert!(matches!(decoded[0], OpCode::Nop));
    assert!(matches!(decoded[1], OpCode::PushInt(100)));
    assert!(matches!(decoded[2], OpCode::PushFloat(value) if value == 25.5));
    assert!(matches!(decoded[3], OpCode::PushBool(true)));
    assert!(matches!(decoded[4], OpCode::PushChar('A')));
    assert!(matches!(decoded[5], OpCode::PushNull));
    assert!(matches!(&decoded[6], OpCode::PushString(value) if value == "Astravon"));
    assert!(matches!(decoded[7], OpCode::Print));

    assert!(matches!(&decoded[8], OpCode::LoadVar(name) if name == "age"));
    assert!(matches!(&decoded[9], OpCode::StoreVar(name) if name == "age"));

    assert!(matches!(decoded[10], OpCode::Add));
    assert!(matches!(decoded[11], OpCode::Subtract));
    assert!(matches!(decoded[12], OpCode::Multiply));
    assert!(matches!(decoded[13], OpCode::Divide));
    assert!(matches!(decoded[14], OpCode::Modulo));

    assert!(matches!(decoded[15], OpCode::Equal));
    assert!(matches!(decoded[16], OpCode::NotEqual));
    assert!(matches!(decoded[17], OpCode::LessThan));
    assert!(matches!(decoded[18], OpCode::GreaterThan));
    assert!(matches!(decoded[19], OpCode::LessThanOrEqual));
    assert!(matches!(decoded[20], OpCode::GreaterThanOrEqual));

    assert!(matches!(decoded[21], OpCode::And));
    assert!(matches!(decoded[22], OpCode::Or));
    assert!(matches!(decoded[23], OpCode::Negate));
    assert!(matches!(decoded[24], OpCode::Not));
}

// ============================================
// 4. RUNTIME ARITHMETIC SAFETY
// ============================================

#[test]
fn test_integer_division_by_zero() {
    let result = run_program(vec![
        OpCode::PushInt(10),
        OpCode::PushInt(0),
        OpCode::Divide,
    ]);

    assert!(result.is_err());
}

#[test]
fn test_integer_modulo_by_zero() {
    let result = run_program(vec![
        OpCode::PushInt(10),
        OpCode::PushInt(0),
        OpCode::Modulo,
    ]);

    assert!(result.is_err());
}

#[test]
fn test_integer_addition_overflow() {
    let result = run_program(vec![
        OpCode::PushInt(i64::MAX),
        OpCode::PushInt(1),
        OpCode::Add,
    ]);

    assert!(result.is_err());
}

#[test]
fn test_integer_subtraction_overflow() {
    let result = run_program(vec![
        OpCode::PushInt(i64::MIN),
        OpCode::PushInt(1),
        OpCode::Subtract,
    ]);

    assert!(result.is_err());
}

#[test]
fn test_integer_multiplication_overflow() {
    let result = run_program(vec![
        OpCode::PushInt(i64::MAX),
        OpCode::PushInt(2),
        OpCode::Multiply,
    ]);

    assert!(result.is_err());
}

#[test]
fn test_integer_negation_overflow() {
    let result = run_program(vec![OpCode::PushInt(i64::MIN), OpCode::Negate]);

    assert!(result.is_err());
}

#[test]
fn test_float_division_by_zero() {
    let result = run_program(vec![
        OpCode::PushFloat(10.0),
        OpCode::PushFloat(0.0),
        OpCode::Divide,
    ]);

    assert!(result.is_err());
}

// ============================================
// 5. RUNTIME STACK SAFETY
// ============================================

#[test]
fn test_addition_stack_underflow() {
    let result = run_program(vec![OpCode::PushInt(10), OpCode::Add]);

    assert!(result.is_err());
}

#[test]
fn test_print_stack_underflow() {
    let result = run_program(vec![OpCode::Print]);

    assert!(result.is_err());
}

#[test]
fn test_negation_stack_underflow() {
    let result = run_program(vec![OpCode::Negate]);

    assert!(result.is_err());
}

// ============================================
// 6. VARIABLE SAFETY
// ============================================

#[test]
fn test_undefined_variable_access() {
    let result = run_program(vec![OpCode::LoadVar("missing_variable".to_string())]);

    assert!(result.is_err());
}

// ============================================
// 7. TYPE SAFETY
// ============================================

#[test]
fn test_arithmetic_type_mismatch() {
    let result = run_program(vec![
        OpCode::PushBool(true),
        OpCode::PushInt(10),
        OpCode::Add,
    ]);

    assert!(result.is_err());
}

#[test]
fn test_logical_type_mismatch() {
    let result = run_program(vec![
        OpCode::PushInt(10),
        OpCode::PushBool(true),
        OpCode::And,
    ]);

    assert!(result.is_err());
}

#[test]
fn test_negation_type_mismatch() {
    let result = run_program(vec![OpCode::PushBool(true), OpCode::Negate]);

    assert!(result.is_err());
}

// ============================================
// END OF ROBUSTNESS TESTS
// ============================================
