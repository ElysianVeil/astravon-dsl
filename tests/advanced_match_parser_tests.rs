use astravon_dsl::ast::nodes::{Expression, MatchPattern};
use astravon_dsl::lexer::tokenizer::Lexer;
use astravon_dsl::parser::grammar::parse_expression;

fn parse(source: &str) -> Expression {
    let tokens = Lexer::tokenize(source).expect("Tokenization failed");
    parse_expression(tokens).expect("Parsing failed")
}

#[test]
fn test_or_pattern() {
    let expression = parse(
        r#"
        match value {
            1 | 2 | 3 => "Small",
            _ => "Other"
        }
        "#,
    );

    let Expression::Match { arms, .. } = expression else {
        panic!("Expected match expression");
    };

    assert_eq!(arms.len(), 2);

    assert_eq!(
        arms[0].pattern,
        MatchPattern::Or(vec![
            MatchPattern::Int(1),
            MatchPattern::Int(2),
            MatchPattern::Int(3),
        ])
    );

    assert!(arms[0].guard.is_none());
}

#[test]
fn test_exclusive_range_pattern() {
    let expression = parse(
        r#"
        match value {
            1..10 => "Range",
            _ => "Other"
        }
        "#,
    );

    let Expression::Match { arms, .. } = expression else {
        panic!("Expected match expression");
    };

    assert_eq!(
        arms[0].pattern,
        MatchPattern::Range {
            start: Box::new(MatchPattern::Int(1)),
            end: Box::new(MatchPattern::Int(10)),
            inclusive: false,
        }
    );
}

#[test]
fn test_inclusive_range_pattern() {
    let expression = parse(
        r#"
        match value {
            1..=10 => "Range",
            _ => "Other"
        }
        "#,
    );

    let Expression::Match { arms, .. } = expression else {
        panic!("Expected match expression");
    };

    assert_eq!(
        arms[0].pattern,
        MatchPattern::Range {
            start: Box::new(MatchPattern::Int(1)),
            end: Box::new(MatchPattern::Int(10)),
            inclusive: true,
        }
    );
}

#[test]
fn test_match_guard() {
    let expression = parse(
        r#"
        match score {
            1..=100 if score >= 50 => "Pass",
            _ => "Fail"
        }
        "#,
    );

    let Expression::Match { arms, .. } = expression else {
        panic!("Expected match expression");
    };

    assert_eq!(arms.len(), 2);
    assert!(arms[0].guard.is_some());
    assert!(arms[1].guard.is_none());
}

#[test]
fn test_or_pattern_with_guard() {
    let expression = parse(
        r#"
        match value {
            1 | 2 | 3 if value > 1 => "Selected",
            _ => "Other"
        }
        "#,
    );

    let Expression::Match { arms, .. } = expression else {
        panic!("Expected match expression");
    };

    assert!(matches!(arms[0].pattern, MatchPattern::Or(_)));

    assert!(arms[0].guard.is_some());
}

#[test]
fn test_float_range_pattern() {
    let expression = parse(
        r#"
        match value {
            1.5..=5.5 => "Range",
            _ => "Other"
        }
        "#,
    );

    let Expression::Match { arms, .. } = expression else {
        panic!("Expected match expression");
    };

    assert_eq!(
        arms[0].pattern,
        MatchPattern::Range {
            start: Box::new(MatchPattern::Float(1.5)),
            end: Box::new(MatchPattern::Float(5.5)),
            inclusive: true,
        }
    );
}

#[test]
fn test_logical_or_is_not_pattern_or() {
    let expression = parse(
        r#"
        match value {
            1 | 2 if value == 1 || value == 2 => "Selected",
            _ => "Other"
        }
        "#,
    );

    let Expression::Match { arms, .. } = expression else {
        panic!("Expected match expression");
    };

    assert!(arms[0].guard.is_some());
}
