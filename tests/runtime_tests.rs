use astravon_dsl::runtime::executor;

#[test]
fn test_hello_world_program() {
    let code = r#"
    print("Hello Astravon!");
    "#;
    let output = executor::run(code).unwrap();
    assert_eq!(output, "Hello Astravon!");
}
