use astravon_dsl::ast::nodes::{
    BinaryOperator, Expression, Program, Statement, TypeAnnotation, UnaryOperator,
};
use astravon_dsl::semantics::analyzer::{Analyzer, SemanticError, Type};

// ============================================
// TEST HELPERS
// ============================================

fn integer(value: i64) -> Expression {
    Expression::IntegerLiteral(value)
}

fn float(value: f64) -> Expression {
    Expression::FloatLiteral(value)
}

fn boolean(value: bool) -> Expression {
    Expression::BooleanLiteral(value)
}

fn string(value: &str) -> Expression {
    Expression::StringLiteral(value.to_string())
}

fn identifier(name: &str) -> Expression {
    Expression::Identifier(name.to_string())
}

fn binary(left: Expression, operator: BinaryOperator, right: Expression) -> Expression {
    Expression::Binary {
        left: Box::new(left),
        operator,
        right: Box::new(right),
    }
}

fn unary(operator: UnaryOperator, operand: Expression) -> Expression {
    Expression::Unary {
        operator,
        operand: Box::new(operand),
    }
}

fn declaration(name: &str, initializer: Expression) -> Statement {
    Statement::Let {
        name: name.to_string(),
        type_annotation: None,
        initializer,
    }
}

fn typed_declaration(name: &str, annotation: TypeAnnotation, initializer: Expression) -> Statement {
    Statement::Let {
        name: name.to_string(),
        type_annotation: Some(annotation),
        initializer,
    }
}

fn assignment(name: &str, value: Expression) -> Statement {
    Statement::Assignment {
        name: name.to_string(),
        value,
    }
}

fn program(statements: Vec<Statement>) -> Program {
    Program { statements }
}

// ============================================
// 1. VALID DECLARATIONS
// ============================================

#[test]
fn test_integer_declaration() {
    let source = program(vec![declaration("age", integer(20))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
    assert_eq!(analyzer.symbols["age"].ty, Type::Int);
}

#[test]
fn test_float_declaration() {
    let source = program(vec![declaration("height", float(1.68))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
    assert_eq!(analyzer.symbols["height"].ty, Type::Float);
}

#[test]
fn test_boolean_declaration() {
    let source = program(vec![declaration("active", boolean(true))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
    assert_eq!(analyzer.symbols["active"].ty, Type::Bool);
}

#[test]
fn test_string_declaration() {
    let source = program(vec![declaration("name", string("Astravon"))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
    assert_eq!(analyzer.symbols["name"].ty, Type::String);
}

// ============================================
// 2. TYPE ANNOTATIONS
// ============================================

#[test]
fn test_valid_integer_annotation() {
    let source = program(vec![typed_declaration(
        "age",
        TypeAnnotation::Int,
        integer(20),
    )]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_valid_float_annotation() {
    let source = program(vec![typed_declaration(
        "height",
        TypeAnnotation::Float,
        float(1.75),
    )]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_invalid_type_annotation() {
    let source = program(vec![typed_declaration(
        "age",
        TypeAnnotation::Int,
        string("twenty"),
    )]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::TypeMismatch { .. })
    ));
}

// ============================================
// 3. VARIABLE SAFETY
// ============================================

#[test]
fn test_duplicate_variable_declaration() {
    let source = program(vec![
        declaration("age", integer(20)),
        declaration("age", integer(25)),
    ]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::DuplicateVariable(name))
            if name == "age"
    ));
}

#[test]
fn test_undeclared_variable_access() {
    let source = program(vec![Statement::Print(identifier("missing"))]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::UndeclaredVariable(name))
            if name == "missing"
    ));
}

#[test]
fn test_assignment_to_undeclared_variable() {
    let source = program(vec![assignment("score", integer(100))]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::UndeclaredVariable(name))
            if name == "score"
    ));
}

#[test]
fn test_valid_variable_assignment() {
    let source = program(vec![
        declaration("score", integer(50)),
        assignment("score", integer(75)),
    ]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_invalid_variable_assignment() {
    let source = program(vec![
        declaration("score", integer(50)),
        assignment("score", string("seventy-five")),
    ]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::TypeMismatch { .. })
    ));
}

// ============================================
// 4. ARITHMETIC TYPE CHECKING
// ============================================

#[test]
fn test_valid_integer_addition() {
    let source = program(vec![Statement::Print(binary(
        integer(10),
        BinaryOperator::Add,
        integer(20),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_valid_mixed_numeric_arithmetic() {
    let source = program(vec![Statement::Print(binary(
        integer(10),
        BinaryOperator::Add,
        float(2.5),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_reject_boolean_arithmetic() {
    let source = program(vec![Statement::Print(binary(
        boolean(true),
        BinaryOperator::Add,
        integer(10),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::UnsupportedExpression(_))
    ));
}

#[test]
fn test_reject_string_arithmetic() {
    let source = program(vec![Statement::Print(binary(
        string("Astravon"),
        BinaryOperator::Multiply,
        integer(2),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::UnsupportedExpression(_))
    ));
}

// ============================================
// 5. COMPARISON TYPE CHECKING
// ============================================

#[test]
fn test_valid_numeric_comparison() {
    let source = program(vec![Statement::Print(binary(
        integer(10),
        BinaryOperator::LessThan,
        integer(20),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_valid_numeric_equality() {
    let source = program(vec![Statement::Print(binary(
        integer(10),
        BinaryOperator::Equal,
        float(10.0),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_reject_string_ordering_comparison() {
    let source = program(vec![Statement::Print(binary(
        string("Astravon"),
        BinaryOperator::LessThan,
        string("Virtual Machine"),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::UnsupportedExpression(_))
    ));
}

#[test]
fn test_reject_incompatible_equality() {
    let source = program(vec![Statement::Print(binary(
        integer(10),
        BinaryOperator::Equal,
        boolean(true),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::TypeMismatch { .. })
    ));
}

// ============================================
// 6. LOGICAL TYPE CHECKING
// ============================================

#[test]
fn test_valid_logical_and() {
    let source = program(vec![Statement::Print(binary(
        boolean(true),
        BinaryOperator::And,
        boolean(false),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_valid_logical_or() {
    let source = program(vec![Statement::Print(binary(
        boolean(true),
        BinaryOperator::Or,
        boolean(false),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_reject_integer_logical_operation() {
    let source = program(vec![Statement::Print(binary(
        integer(10),
        BinaryOperator::And,
        boolean(true),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::TypeMismatch { .. })
    ));
}

// ============================================
// 7. UNARY TYPE CHECKING
// ============================================

#[test]
fn test_valid_integer_negation() {
    let source = program(vec![Statement::Print(unary(
        UnaryOperator::Negate,
        integer(10),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_valid_boolean_not() {
    let source = program(vec![Statement::Print(unary(
        UnaryOperator::Not,
        boolean(true),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_reject_string_negation() {
    let source = program(vec![Statement::Print(unary(
        UnaryOperator::Negate,
        string("Astravon"),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::UnsupportedExpression(_))
    ));
}

#[test]
fn test_reject_integer_logical_not() {
    let source = program(vec![Statement::Print(unary(
        UnaryOperator::Not,
        integer(10),
    ))]);

    let mut analyzer = Analyzer::new();

    assert!(matches!(
        analyzer.analyze_program(&source),
        Err(SemanticError::TypeMismatch { .. })
    ));
}

// ============================================
// 8. COMPLEX EXPRESSIONS
// ============================================

#[test]
fn test_valid_nested_expression() {
    let expression = binary(
        binary(integer(10), BinaryOperator::Add, integer(5)),
        BinaryOperator::Multiply,
        integer(2),
    );

    let source = program(vec![Statement::Print(expression)]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}

#[test]
fn test_valid_variable_expression() {
    let source = program(vec![
        declaration("age", integer(20)),
        declaration(
            "next_age",
            binary(identifier("age"), BinaryOperator::Add, integer(1)),
        ),
    ]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
    assert_eq!(analyzer.symbols["next_age"].ty, Type::Int);
}

// ============================================
// 9. MULTIPLE STATEMENTS
// ============================================

#[test]
fn test_valid_multi_statement_program() {
    let source = program(vec![
        declaration("age", integer(20)),
        declaration("active", boolean(true)),
        assignment(
            "age",
            binary(identifier("age"), BinaryOperator::Add, integer(1)),
        ),
        Statement::Print(identifier("age")),
        Statement::Print(identifier("active")),
    ]);

    let mut analyzer = Analyzer::new();

    assert!(analyzer.analyze_program(&source).is_ok());
}
