use astravon_dsl::bytecode::{decoder, encoder};
use astravon_dsl::runtime::OpCode;

#[test]
fn test_bytecode_header() {
    let bytecode = encoder::encode(&[OpCode::Nop]).unwrap();

    assert_eq!(&bytecode[0..3], b"AVM");
    assert_eq!(bytecode[3], 4);
}

#[test]
fn test_bytecode_round_trip() {
    let instructions = vec![
        OpCode::Nop,
        OpCode::PushInt(42),
        OpCode::PushFloat(3.14),
        OpCode::PushBool(true),
        OpCode::PushChar('A'),
        OpCode::PushNull,
        OpCode::PushString("Astravon".to_string()),
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
        OpCode::Print,
    ];

    let bytecode = encoder::encode(&instructions).unwrap();
    let decoded = decoder::decode(&bytecode).unwrap();

    assert_eq!(instructions.len(), decoded.len());

    // Verify individual instructions below because OpCode
    // may not yet implement PartialEq.
    assert!(matches!(decoded[0], OpCode::Nop));
    assert!(matches!(decoded[1], OpCode::PushInt(42)));
    assert!(matches!(decoded[3], OpCode::PushBool(true)));
    assert!(matches!(decoded[7], OpCode::LoadVar(ref name) if name == "age"));
    assert!(matches!(decoded[8], OpCode::StoreVar(ref name) if name == "age"));
    assert!(matches!(decoded[24], OpCode::Print));
}

#[test]
fn test_invalid_bytecode_header() {
    let result = decoder::decode(b"XYZ\x03");

    assert!(result.is_err());
}

#[test]
fn test_truncated_integer_operand() {
    let bytecode = b"AVM\x01\x01\x01\x02";

    assert!(decoder::decode(bytecode).is_err());
}

#[test]
fn test_unknown_opcode() {
    let bytecode = b"AVM\x01\xFF";

    assert!(decoder::decode(bytecode).is_err());
}

#[test]
fn test_jump_if_false_bytecode_round_trip() {
    let instructions = vec![
        OpCode::PushBool(false),
        OpCode::JumpIfFalse(3),
        OpCode::PushString("Skipped".to_string()),
        OpCode::Print,
    ];

    let encoded = encoder::encode(&instructions).unwrap();
    let decoded = decoder::decode(&encoded).unwrap();

    assert_eq!(decoded, instructions);
    assert_eq!(&encoded[0..3], b"AVM");
    assert_eq!(encoded[3], 4);
}
