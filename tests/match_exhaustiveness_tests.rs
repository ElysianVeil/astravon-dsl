use astravon_dsl::{lexer::tokenizer::Lexer, parser::grammar, semantics::analyzer};

fn analyze_program(source: &str) -> Result<(), String> {
    let tokens = Lexer::tokenize(source)?;

    let program = grammar::parse_program(tokens).map_err(|error| format!("{error:?}"))?;

    let mut analyzer = analyzer::Analyzer::new();
    analyzer
        .analyze_program(&program)
        .map_err(|error| format!("{error:?}"))?;

    Ok(())
}

#[test]
fn test_boolean_match_is_exhaustive() {
    let source = r#"
        let enabled = true;

        let result = match enabled {
            true => "Active",
            false => "Inactive"
        };

        print result;
    "#;

    assert!(analyze_program(source).is_ok());
}

#[test]
fn test_boolean_match_with_wildcard_is_exhaustive() {
    let source = r#"
        let enabled = true;

        let result = match enabled {
            true => "Active",
            _ => "Inactive"
        };
    "#;

    assert!(analyze_program(source).is_ok());
}

#[test]
fn test_integer_match_without_wildcard_is_rejected() {
    let source = r#"
        let score = 80;

        let result = match score {
            80 => "B",
            90 => "A"
        };
    "#;

    assert!(analyze_program(source).is_err());
}

#[test]
fn test_integer_match_with_wildcard_is_exhaustive() {
    let source = r#"
        let score = 80;

        let result = match score {
            80 => "B",
            90 => "A",
            _ => "Other"
        };
    "#;

    assert!(analyze_program(source).is_ok());
}

#[test]
fn test_incomplete_boolean_match_is_rejected() {
    let source = r#"
        let enabled = true;

        let result = match enabled {
            true => "Active"
        };
    "#;

    assert!(analyze_program(source).is_err());
}

#[test]
fn test_duplicate_pattern_is_rejected() {
    let source = r#"
        let score = 80;

        let result = match score {
            80 => "B",
            80 => "Other",
            _ => "Unknown"
        };
    "#;

    assert!(analyze_program(source).is_err());
}

#[test]
fn test_pattern_after_wildcard_is_rejected() {
    let source = r#"
        let score = 80;

        let result = match score {
            _ => "Unknown",
            80 => "B"
        };
    "#;

    assert!(analyze_program(source).is_err());
}

#[test]
fn test_pattern_type_mismatch_is_rejected() {
    let source = r#"
        let score = 80;

        let result = match score {
            true => "Active",
            _ => "Other"
        };
    "#;

    assert!(analyze_program(source).is_err());
}

#[test]
fn test_match_arm_result_type_mismatch_is_rejected() {
    let source = r#"
        let score = 80;

        let result = match score {
            80 => "B",
            _ => 0
        };
    "#;

    assert!(analyze_program(source).is_err());
}

#[test]
fn test_empty_match_is_rejected() {
    let source = r#"
        let score = 80;
        let result = match score {};
    "#;

    assert!(analyze_program(source).is_err());
}
