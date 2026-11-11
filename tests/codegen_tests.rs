use astravon_dsl::ast::nodes::{Expression, Program, Statement};
use astravon_dsl::codegen::llvm_backend;

#[test]
fn test_llvm_ir_generation() {
    let ast = Program {
        statements: vec![Statement::Print(Expression::StringLiteral(
            "Hello Astravon!".to_string(),
        ))],
    };

    let ir = llvm_backend::generate_ir(&ast).unwrap();

    assert!(ir.contains("define i32 @main"));
    assert!(ir.contains("Hello Astravon!"));
    assert!(ir.contains("puts"));
}
