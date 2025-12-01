use astravon_dsl::codegen::llvm_backend;

#[test]
fn test_llvm_ir_generation() {
    let ast = /* build small AST */;
    let ir = llvm_backend::generate_ir(ast).unwrap();
    assert!(ir.contains("define i32 @main"));
}
