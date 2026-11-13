use astravon_dsl::ast::nodes::{Expression, Parameter, Statement, TypeAnnotation};
use astravon_dsl::semantics::analyzer::{Analyzer, SemanticError};

fn parameter(name: &str, ty: TypeAnnotation) -> Parameter {
    Parameter {
        name: name.to_string(),
        type_annotation: ty,
    }
}

fn return_int(value: i64) -> Statement {
    Statement::Return(Some(Expression::IntegerLiteral(value)))
}

fn condition() -> Expression {
    Expression::Identifier("value".to_string())
}

fn function(body: Vec<Statement>) -> Statement {
    Statement::Function {
        name: "calculate".to_string(),
        parameters: vec![parameter("value", TypeAnnotation::Bool)],
        return_type: Some(TypeAnnotation::Int),
        body,
    }
}

fn analyze_function_body(body: Vec<Statement>) -> Result<(), SemanticError> {
    let program = astravon_dsl::ast::nodes::Program {
        statements: vec![function(body)],
    };

    Analyzer::new().analyze_program(&program)
}

#[test]
fn test_direct_return_is_accepted() {
    assert!(analyze_function_body(vec![return_int(10),]).is_ok());
}

#[test]
fn test_if_else_both_return_is_accepted() {
    assert!(
        analyze_function_body(vec![Statement::If {
            condition: condition(),
            then_branch: vec![return_int(10)],
            else_branch: Some(vec![return_int(20)]),
        },])
        .is_ok()
    );
}

#[test]
fn test_if_without_else_is_rejected() {
    let result = analyze_function_body(vec![Statement::If {
        condition: condition(),
        then_branch: vec![return_int(10)],
        else_branch: None,
    }]);

    assert!(matches!(result, Err(SemanticError::MissingReturn(_))));
}

#[test]
fn test_if_else_missing_return_in_one_branch_is_rejected() {
    let result = analyze_function_body(vec![Statement::If {
        condition: condition(),
        then_branch: vec![return_int(10)],
        else_branch: Some(vec![Statement::Print(Expression::IntegerLiteral(20))]),
    }]);

    assert!(matches!(result, Err(SemanticError::MissingReturn(_))));
}

#[test]
fn test_else_if_chain_with_all_returns_is_accepted() {
    let nested_if = Statement::If {
        condition: condition(),
        then_branch: vec![return_int(20)],
        else_branch: Some(vec![return_int(30)]),
    };

    assert!(
        analyze_function_body(vec![Statement::If {
            condition: condition(),
            then_branch: vec![return_int(10)],
            else_branch: Some(vec![nested_if]),
        },])
        .is_ok()
    );
}

#[test]
fn test_nested_conditional_missing_return_is_rejected() {
    let nested_if = Statement::If {
        condition: condition(),
        then_branch: vec![return_int(20)],
        else_branch: None,
    };

    let result = analyze_function_body(vec![Statement::If {
        condition: condition(),
        then_branch: vec![nested_if],
        else_branch: Some(vec![return_int(30)]),
    }]);

    assert!(matches!(result, Err(SemanticError::MissingReturn(_))));
}

#[test]
fn test_conditional_return_followed_by_final_return_is_accepted() {
    assert!(
        analyze_function_body(vec![
            Statement::If {
                condition: condition(),
                then_branch: vec![return_int(10)],
                else_branch: None,
            },
            return_int(20),
        ])
        .is_ok()
    );
}

#[test]
fn test_void_function_does_not_require_return() {
    let program = astravon_dsl::ast::nodes::Program {
        statements: vec![Statement::Function {
            name: "display".to_string(),
            parameters: vec![],
            return_type: None,
            body: vec![Statement::Print(Expression::StringLiteral(
                "Hello Astravon".to_string(),
            ))],
        }],
    };

    assert!(Analyzer::new().analyze_program(&program).is_ok());
}
