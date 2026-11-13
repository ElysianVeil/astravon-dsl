// src/semantics/analyzer.rs

use crate::ast::nodes::{
    BinaryOperator, Expression, MatchArm, MatchPattern, Parameter, Program, Statement,
    StringSegment, TypeAnnotation, UnaryOperator,
};

use std::collections::HashMap;

/// Represents the type of a value in Astravon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    String,
    Int,
    Float,
    Bool,
    Char,
    Null,
    Array(Box<Type>),
    Void,
    Unknown,
}

/// Represents a variable in a symbol table.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
}

/// Represents a registered function signature.
#[derive(Debug, Clone)]
struct FunctionSignature {
    parameters: Vec<Type>,
    return_type: Type,
}

/// Semantic analyzer state.
pub struct Analyzer {
    /// Global variables.
    pub symbols: HashMap<String, Symbol>,

    /// Registered function signatures.
    functions: HashMap<String, FunctionSignature>,

    /// Local variable scopes.
    scopes: Vec<HashMap<String, Symbol>>,

    /// Return type of the function currently being analyzed.
    current_return_type: Option<Type>,

    loop_depth: usize,
}

#[derive(Debug)]
pub enum SemanticError {
    UndeclaredVariable(String),

    DuplicateVariable(String),

    DuplicateFunction(String),

    TypeMismatch {
        expected: Type,
        found: Type,
    },

    UnsupportedExpression(String),

    UndefinedFunction(String),

    ArgumentCountMismatch {
        function: String,
        expected: usize,
        found: usize,
    },

    InvalidReturn(String),

    MissingReturn(String),
}

pub type SemanticResult<T> = Result<T, SemanticError>;

impl Analyzer {
    /// Create a new semantic analyzer.
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            functions: HashMap::new(),
            scopes: Vec::new(),
            current_return_type: None,
            loop_depth: 0,
        }
    }

    /// Analyze an entire program.
    pub fn analyze_program(&mut self, program: &Program) -> SemanticResult<()> {
        // Register every function before analyzing any expressions.
        // This permits forward calls and recursion.
        self.register_functions(program)?;

        // Analyze global statements first.
        // Function bodies are analyzed after global declarations exist.
        for statement in &program.statements {
            if !matches!(statement, Statement::Function { .. }) {
                self.analyze_statement(statement)?;
            }
        }

        // Analyze function bodies.
        for statement in &program.statements {
            if let Statement::Function {
                name,
                parameters,
                return_type,
                body,
            } = statement
            {
                self.analyze_function(name, parameters, return_type, body)?;
            }
        }

        Ok(())
    }

    /// Register function signatures before analyzing function bodies.
    fn register_functions(&mut self, program: &Program) -> SemanticResult<()> {
        for statement in &program.statements {
            if let Statement::Function {
                name,
                parameters,
                return_type,
                ..
            } = statement
            {
                if self.functions.contains_key(name) || self.symbols.contains_key(name) {
                    return Err(SemanticError::DuplicateFunction(name.clone()));
                }

                let mut parameter_types = Vec::with_capacity(parameters.len());
                let mut parameter_names = std::collections::HashSet::new();

                for parameter in parameters {
                    if !parameter_names.insert(parameter.name.clone()) {
                        return Err(SemanticError::DuplicateVariable(parameter.name.clone()));
                    }

                    parameter_types.push(Self::annotation_to_type(&parameter.type_annotation));
                }

                let function_return_type = match return_type {
                    Some(annotation) => Self::annotation_to_type(annotation),
                    None => Type::Void,
                };

                self.functions.insert(
                    name.clone(),
                    FunctionSignature {
                        parameters: parameter_types,
                        return_type: function_return_type,
                    },
                );
            }
        }

        Ok(())
    }

    /// Analyze a function declaration and its body.
    fn analyze_function(
        &mut self,
        name: &str,
        parameters: &[Parameter],
        return_type: &Option<TypeAnnotation>,
        body: &[Statement],
    ) -> SemanticResult<()> {
        let expected_return_type = match return_type {
            Some(annotation) => Self::annotation_to_type(annotation),
            None => Type::Void,
        };

        // Each function receives its own local scope.
        self.scopes.push(HashMap::new());
        self.current_return_type = Some(expected_return_type.clone());

        // Analyze parameters and the function body.
        let analysis_result = (|| {
            for parameter in parameters {
                let parameter_type = Self::annotation_to_type(&parameter.type_annotation);

                self.declare_local(&parameter.name, parameter_type)?;
            }

            for statement in body {
                // Nested function declarations are not supported yet.
                if matches!(statement, Statement::Function { .. }) {
                    return Err(SemanticError::UnsupportedExpression(
                        "Nested function declarations are not supported".into(),
                    ));
                }

                self.analyze_statement(statement)?;
            }

            // Every execution path in a non-void function must return.
            if expected_return_type != Type::Void && !Self::block_guarantees_return(body) {
                return Err(SemanticError::MissingReturn(name.to_string()));
            }

            Ok(())
        })();

        // Always restore analyzer state, including when analysis fails.
        self.scopes.pop();
        self.current_return_type = None;

        analysis_result
    }

    /// Determine whether a sequence of statements guarantees a return.
    ///
    /// A block guarantees a return if at least one statement in its
    /// sequence guarantees a return on every path reaching that statement.
    fn block_guarantees_return(statements: &[Statement]) -> bool {
        statements.iter().any(Self::statement_guarantees_return)
    }

    /// Determine whether an individual statement guarantees a return.
    fn statement_guarantees_return(statement: &Statement) -> bool {
        match statement {
            // A return statement always exits the current function.
            Statement::Return(_) => true,

            // Both branches must guarantee a return.
            Statement::If {
                then_branch,
                else_branch: Some(else_branch),
                ..
            } => {
                Self::block_guarantees_return(then_branch)
                    && Self::block_guarantees_return(else_branch)
            }

            // Without an else branch, the condition could be false.
            Statement::If {
                else_branch: None, ..
            } => false,

            // Other statements do not independently guarantee a return.
            _ => false,
        }
    }

    /// Analyze an individual statement.
    fn analyze_statement(&mut self, statement: &Statement) -> SemanticResult<()> {
        match statement {
            Statement::Let {
                name,
                type_annotation,
                initializer,
            } => {
                if self.functions.contains_key(name) {
                    return Err(SemanticError::DuplicateVariable(name.clone()));
                }

                let inferred_type = self.analyze_expression(initializer)?;

                if inferred_type == Type::Void {
                    return Err(SemanticError::UnsupportedExpression(
                        "A void function cannot initialize a variable".into(),
                    ));
                }

                let declared_type = match type_annotation {
                    Some(annotation) => Self::annotation_to_type(annotation),
                    None => inferred_type.clone(),
                };

                Self::check_type_compatibility(&declared_type, &inferred_type)?;

                self.declare_variable(name, declared_type)
            }

            Statement::Assignment { name, value } => {
                let expected_type = self
                    .lookup_variable(name)
                    .ok_or_else(|| SemanticError::UndeclaredVariable(name.clone()))?;

                let found_type = self.analyze_expression(value)?;

                Self::check_type_compatibility(&expected_type, &found_type)
            }

            Statement::Print(expression) => {
                let expression_type = self.analyze_expression(expression)?;

                if expression_type == Type::Void {
                    return Err(SemanticError::UnsupportedExpression(
                        "Cannot print the result of a void function".into(),
                    ));
                }

                Ok(())
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Conditions must evaluate to Bool.
                let condition_type = self.analyze_expression(condition)?;

                if condition_type != Type::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Type::Bool,
                        found: condition_type,
                    });
                }

                // Analyze the true branch in its own lexical scope.
                self.analyze_block_in_scope(then_branch)?;

                // Analyze the optional false branch in a separate scope.
                if let Some(else_branch) = else_branch {
                    self.analyze_block_in_scope(else_branch)?;
                }

                Ok(())
            }

            Statement::While { condition, body } => {
                // A while-loop condition must evaluate to Bool.
                let condition_type = self.analyze_expression(condition)?;

                if condition_type != Type::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Type::Bool,
                        found: condition_type,
                    });
                }

                // Enter loop context.
                self.loop_depth += 1;

                // Analyze the loop body in its own lexical scope.
                let analysis_result = self.analyze_block_in_scope(body);

                // Always restore loop depth.
                self.loop_depth -= 1;

                analysis_result
            }

            Statement::For {
                variable,
                iterable,
                body,
            } => {
                let iterable_type = self.analyze_expression(iterable)?;

                let element_type = match iterable_type {
                    Type::Array(element_type) => *element_type,

                    found => {
                        return Err(SemanticError::UnsupportedExpression(format!(
                            "A for-in loop requires an array, found {:?}",
                            found
                        )));
                    }
                };

                // The loop variable exists only within the loop's scope.
                self.scopes.push(HashMap::new());

                let analysis_result = (|| {
                    self.declare_variable(variable, element_type)?;

                    // Enter loop context before analyzing the body.
                    self.loop_depth += 1;

                    let body_result = self.analyze_block_in_scope(body);

                    // Restore loop depth even if the body fails.
                    self.loop_depth -= 1;

                    body_result
                })();

                // Restore lexical scope even when analysis fails.
                self.scopes.pop();

                analysis_result
            }

            Statement::Break => {
                if self.loop_depth == 0 {
                    return Err(SemanticError::UnsupportedExpression(
                        "`break` is only valid inside a loop".into(),
                    ));
                }

                Ok(())
            }

            Statement::Continue => {
                if self.loop_depth == 0 {
                    return Err(SemanticError::UnsupportedExpression(
                        "`continue` is only valid inside a loop".into(),
                    ));
                }

                Ok(())
            }

            Statement::Function { .. } => {
                // Functions are handled during the function-analysis pass.
                Ok(())
            }

            Statement::Expression(expression) => {
                self.analyze_expression(expression)?;
                Ok(())
            }

            Statement::Return(expression) => {
                let expected_type = self.current_return_type.clone().ok_or_else(|| {
                    SemanticError::InvalidReturn("Return statement outside a function".into())
                })?;

                let found_type = match expression {
                    Some(expression) => self.analyze_expression(expression)?,

                    None => Type::Void,
                };

                Self::check_type_compatibility(&expected_type, &found_type)
            }
        }
    }

    /// Analyze a block of statements inside a new lexical scope.
    ///
    /// Variables declared inside this block are not visible outside it.
    /// The scope is removed even when semantic analysis fails.
    fn analyze_block_in_scope(&mut self, statements: &[Statement]) -> SemanticResult<()> {
        self.scopes.push(HashMap::new());

        let result = (|| {
            for statement in statements {
                self.analyze_statement(statement)?;
            }

            Ok(())
        })();

        self.scopes.pop();

        result
    }

    /// Declare a variable in the current scope or global scope.
    fn declare_variable(&mut self, name: &str, ty: Type) -> SemanticResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name) {
                return Err(SemanticError::DuplicateVariable(name.to_string()));
            }

            scope.insert(
                name.to_string(),
                Symbol {
                    name: name.to_string(),
                    ty,
                },
            );
        } else {
            if self.symbols.contains_key(name) {
                return Err(SemanticError::DuplicateVariable(name.to_string()));
            }

            self.symbols.insert(
                name.to_string(),
                Symbol {
                    name: name.to_string(),
                    ty,
                },
            );
        }

        Ok(())
    }

    /// Declare a function parameter in the current local scope.
    fn declare_local(&mut self, name: &str, ty: Type) -> SemanticResult<()> {
        let scope = self.scopes.last_mut().ok_or_else(|| {
            SemanticError::UnsupportedExpression(
                "Cannot declare a local variable without an active scope".into(),
            )
        })?;

        if scope.contains_key(name) {
            return Err(SemanticError::DuplicateVariable(name.to_string()));
        }

        scope.insert(
            name.to_string(),
            Symbol {
                name: name.to_string(),
                ty,
            },
        );

        Ok(())
    }

    /// Look up a variable, searching local scopes before global variables.
    fn lookup_variable(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol.ty.clone());
            }
        }

        self.symbols.get(name).map(|symbol| symbol.ty.clone())
    }

    /// Determine the semantic type of an expression.
    fn analyze_expression(&mut self, expression: &Expression) -> SemanticResult<Type> {
        match expression {
            Expression::StringLiteral(_) => Ok(Type::String),

            Expression::InterpolatedString(segments) => {
                for segment in segments {
                    if let StringSegment::Expression(expression) = segment {
                        let ty = self.analyze_expression(expression)?;

                        if ty == Type::Void {
                            return Err(SemanticError::UnsupportedExpression(
                                "Cannot interpolate a void expression".into(),
                            ));
                        }
                    }
                }

                Ok(Type::String)
            }

            Expression::IntegerLiteral(_) => Ok(Type::Int),

            Expression::FloatLiteral(_) => Ok(Type::Float),

            Expression::BooleanLiteral(_) => Ok(Type::Bool),

            Expression::CharacterLiteral(_) => Ok(Type::Char),

            Expression::NullLiteral => Ok(Type::Null),

            Expression::Array(elements) => {
                let Some(first_element) = elements.first() else {
                    return Err(SemanticError::UnsupportedExpression(
                        "Cannot infer the element type of an empty array".into(),
                    ));
                };

                let element_type = self.analyze_expression(first_element)?;

                if element_type == Type::Void {
                    return Err(SemanticError::UnsupportedExpression(
                        "An array cannot contain a void expression".into(),
                    ));
                }

                for element in elements.iter().skip(1) {
                    let found_type = self.analyze_expression(element)?;

                    // Arrays require exact homogeneous element types.
                    // Int and Float are deliberately not interchangeable here.
                    if found_type != element_type {
                        return Err(SemanticError::TypeMismatch {
                            expected: element_type.clone(),
                            found: found_type,
                        });
                    }
                }

                Ok(Type::Array(Box::new(element_type)))
            }

            Expression::Match { value, arms } => self.analyze_match_expression(value, arms),

            Expression::Identifier(name) => self
                .lookup_variable(name)
                .ok_or_else(|| SemanticError::UndeclaredVariable(name.clone())),

            Expression::Call { name, arguments } => self.analyze_function_call(name, arguments),

            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                self.analyze_binary_operator(operator, left_type, right_type)
            }

            Expression::Unary { operator, operand } => {
                let operand_type = self.analyze_expression(operand)?;

                match operator {
                    UnaryOperator::Negate => {
                        if Self::is_numeric(&operand_type) {
                            Ok(operand_type)
                        } else {
                            Err(SemanticError::UnsupportedExpression(format!(
                                "Unary negation requires a numeric operand, found {:?}",
                                operand_type
                            )))
                        }
                    }

                    UnaryOperator::Not => {
                        if operand_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                expected: Type::Bool,
                                found: operand_type,
                            })
                        }
                    }
                }
            }
        }
    }

    fn analyze_match_expression(
        &mut self,
        value: &Expression,
        arms: &[MatchArm],
    ) -> SemanticResult<Type> {
        if arms.is_empty() {
            return Err(SemanticError::UnsupportedExpression(
                "Match expression requires at least one arm".to_string(),
            ));
        }

        let value_type = self.analyze_expression(value)?;

        // Track patterns that have already been covered by unguarded arms.
        let mut seen_literals: Vec<MatchPattern> = Vec::new();
        let mut seen_ranges: Vec<MatchPattern> = Vec::new();

        let mut has_wildcard = false;
        let mut has_true = false;
        let mut has_false = false;

        let mut result_type: Option<Type> = None;

        for (index, arm) in arms.iter().enumerate() {
            // Reject arms that can never be reached.
            if has_wildcard || (value_type == Type::Bool && has_true && has_false) {
                return Err(SemanticError::UnsupportedExpression(format!(
                    "Unreachable match arm at position {}: \
                        previous unguarded patterns already cover all values",
                    index + 1
                )));
            }

            // Validate the pattern and ensure it matches the value's type.
            let pattern_type = Self::validate_match_pattern(&arm.pattern)?;

            if let Some(pattern_type) = pattern_type {
                if pattern_type != value_type {
                    return Err(SemanticError::UnsupportedExpression(format!(
                        "Match pattern type mismatch: expected {:?}, found {:?}",
                        value_type, pattern_type
                    )));
                }
            }

            // Guards must evaluate to Bool.
            if let Some(guard) = &arm.guard {
                let guard_type = self.analyze_expression(guard)?;

                if guard_type != Type::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Type::Bool,
                        found: guard_type,
                    });
                }
            }

            let is_unguarded = arm.guard.is_none();

            if is_unguarded {
                let mut literals = Vec::new();
                let mut ranges = Vec::new();

                Self::collect_match_pattern_components(&arm.pattern, &mut literals, &mut ranges);

                // Reject literals already covered by earlier unguarded arms.
                for literal in &literals {
                    if seen_literals.contains(literal) {
                        return Err(SemanticError::UnsupportedExpression(format!(
                            "Duplicate or unreachable match pattern: {:?}",
                            literal
                        )));
                    }
                }

                // Reject repeated ranges.
                for range in &ranges {
                    if seen_ranges.contains(range) {
                        return Err(SemanticError::UnsupportedExpression(format!(
                            "Duplicate match range: {:?}",
                            range
                        )));
                    }
                }

                seen_literals.extend(literals);
                seen_ranges.extend(ranges);

                // Update exhaustiveness information.
                match &arm.pattern {
                    MatchPattern::Wildcard => {
                        has_wildcard = true;
                    }

                    MatchPattern::Bool(true) => {
                        has_true = true;
                    }

                    MatchPattern::Bool(false) => {
                        has_false = true;
                    }

                    MatchPattern::Or(patterns) => {
                        for pattern in patterns {
                            match pattern {
                                MatchPattern::Bool(true) => has_true = true,
                                MatchPattern::Bool(false) => has_false = true,
                                _ => {}
                            }
                        }
                    }

                    _ => {}
                }
            }

            // Analyze the expression produced by this arm.
            let arm_type = self.analyze_expression(&arm.expression)?;

            match &result_type {
                None => {
                    result_type = Some(arm_type);
                }

                Some(expected_type) if *expected_type != arm_type => {
                    return Err(SemanticError::UnsupportedExpression(format!(
                        "Match arms must return compatible types: \
                            expected {:?}, found {:?}",
                        expected_type, arm_type
                    )));
                }

                _ => {}
            }
        }

        let exhaustive = has_wildcard || (value_type == Type::Bool && has_true && has_false);

        if !exhaustive {
            return Err(SemanticError::UnsupportedExpression(format!(
                "Non-exhaustive match expression for type {:?}: \
                    add an unguarded wildcard (_) arm to cover remaining values",
                value_type
            )));
        }

        result_type.ok_or_else(|| {
            SemanticError::UnsupportedExpression("Match expression has no result type".to_string())
        })
    }

    /// Validate a function call and determine its return type.
    fn analyze_function_call(
        &mut self,
        name: &str,
        arguments: &[Expression],
    ) -> SemanticResult<Type> {
        let signature = self
            .functions
            .get(name)
            .cloned()
            .ok_or_else(|| SemanticError::UndefinedFunction(name.to_string()))?;

        if arguments.len() != signature.parameters.len() {
            return Err(SemanticError::ArgumentCountMismatch {
                function: name.to_string(),
                expected: signature.parameters.len(),
                found: arguments.len(),
            });
        }

        for (argument, expected_type) in arguments.iter().zip(signature.parameters.iter()) {
            let found_type = self.analyze_expression(argument)?;

            Self::check_type_compatibility(expected_type, &found_type)?;
        }

        Ok(signature.return_type)
    }

    /// Type-check a binary operation and determine its result type.
    fn analyze_binary_operator(
        &self,
        operator: &BinaryOperator,
        left: Type,
        right: Type,
    ) -> SemanticResult<Type> {
        match operator {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => {
                if !Self::is_numeric(&left) {
                    return Err(SemanticError::UnsupportedExpression(format!(
                        "Arithmetic requires numeric operands, found {:?}",
                        left
                    )));
                }

                if !Self::is_numeric(&right) {
                    return Err(SemanticError::UnsupportedExpression(format!(
                        "Arithmetic requires numeric operands, found {:?}",
                        right
                    )));
                }

                if left == Type::Float || right == Type::Float {
                    Ok(Type::Float)
                } else {
                    Ok(Type::Int)
                }
            }

            BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThanOrEqual => {
                if Self::is_numeric(&left) && Self::is_numeric(&right) {
                    Ok(Type::Bool)
                } else {
                    Err(SemanticError::UnsupportedExpression(
                        "Ordering comparisons require numeric operands".to_string(),
                    ))
                }
            }

            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                if Self::are_compatible(&left, &right) {
                    Ok(Type::Bool)
                } else {
                    Err(SemanticError::TypeMismatch {
                        expected: left,
                        found: right,
                    })
                }
            }

            BinaryOperator::And | BinaryOperator::Or => {
                if left != Type::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Type::Bool,
                        found: left,
                    });
                }

                if right != Type::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: Type::Bool,
                        found: right,
                    });
                }

                Ok(Type::Bool)
            }
        }
    }

    /// Convert an AST type annotation into a semantic type.
    fn annotation_to_type(annotation: &TypeAnnotation) -> Type {
        match annotation {
            TypeAnnotation::Int => Type::Int,
            TypeAnnotation::Float => Type::Float,
            TypeAnnotation::Bool => Type::Bool,
            TypeAnnotation::Char => Type::Char,
            TypeAnnotation::String => Type::String,
            TypeAnnotation::Null => Type::Null,
        }
    }

    fn validate_match_pattern(pattern: &MatchPattern) -> SemanticResult<Option<Type>> {
        match pattern {
            MatchPattern::Int(_) => Ok(Some(Type::Int)),

            MatchPattern::Float(value) => {
                if !value.is_finite() {
                    return Err(SemanticError::UnsupportedExpression(
                        "Match patterns must contain finite floating-point values".to_string(),
                    ));
                }

                Ok(Some(Type::Float))
            }

            MatchPattern::Bool(_) => Ok(Some(Type::Bool)),

            MatchPattern::Char(_) => Ok(Some(Type::Char)),

            MatchPattern::String(_) => Ok(Some(Type::String)),

            MatchPattern::Wildcard => Ok(None),

            MatchPattern::Or(patterns) => {
                if patterns.is_empty() {
                    return Err(SemanticError::UnsupportedExpression(
                        "OR pattern requires at least one alternative".to_string(),
                    ));
                }

                let mut expected_type: Option<Type> = None;
                let mut seen = Vec::new();

                for alternative in patterns {
                    if matches!(alternative, MatchPattern::Wildcard | MatchPattern::Or(_)) {
                        return Err(SemanticError::UnsupportedExpression(
                            "Wildcard and nested OR patterns are not supported \
                            inside OR alternatives"
                                .to_string(),
                        ));
                    }

                    if seen.contains(alternative) {
                        return Err(SemanticError::UnsupportedExpression(format!(
                            "Duplicate alternative in OR pattern: {:?}",
                            alternative
                        )));
                    }

                    seen.push(alternative.clone());

                    let alternative_type = Self::validate_match_pattern(alternative)?;

                    let alternative_type = alternative_type.ok_or_else(|| {
                        SemanticError::UnsupportedExpression(
                            "Invalid wildcard alternative in OR pattern".to_string(),
                        )
                    })?;

                    match &expected_type {
                        None => {
                            expected_type = Some(alternative_type);
                        }

                        Some(expected) if *expected != alternative_type => {
                            return Err(SemanticError::UnsupportedExpression(format!(
                                "OR pattern alternatives must have the same type: \
                                    expected {:?}, found {:?}",
                                expected, alternative_type
                            )));
                        }

                        _ => {}
                    }
                }

                Ok(expected_type)
            }

            MatchPattern::Range {
                start,
                end,
                inclusive: _,
            } => {
                let (range_type, is_reversed) = match (start.as_ref(), end.as_ref()) {
                    (MatchPattern::Int(start), MatchPattern::Int(end)) => (Type::Int, start > end),

                    (MatchPattern::Float(start), MatchPattern::Float(end)) => {
                        if !start.is_finite() || !end.is_finite() {
                            return Err(SemanticError::UnsupportedExpression(
                                "Range boundaries must be finite".to_string(),
                            ));
                        }

                        (Type::Float, start > end)
                    }

                    (MatchPattern::Int(_), MatchPattern::Float(_))
                    | (MatchPattern::Float(_), MatchPattern::Int(_)) => {
                        return Err(SemanticError::UnsupportedExpression(
                            "Range boundaries must have the same numeric type".to_string(),
                        ));
                    }

                    _ => {
                        return Err(SemanticError::UnsupportedExpression(
                            "Range boundaries must be integer or \
                                    floating-point literals"
                                .to_string(),
                        ));
                    }
                };

                if is_reversed {
                    return Err(SemanticError::UnsupportedExpression(
                        "Range start cannot be greater than its end".to_string(),
                    ));
                }

                Ok(Some(range_type))
            }
        }
    }

    /// Collect literal and range components from a match pattern.
    ///
    /// OR patterns are expanded so that duplicates across separate arms
    /// and alternatives can be detected.
    fn collect_match_pattern_components(
        pattern: &MatchPattern,
        literals: &mut Vec<MatchPattern>,
        ranges: &mut Vec<MatchPattern>,
    ) {
        match pattern {
            MatchPattern::Or(patterns) => {
                for alternative in patterns {
                    Self::collect_match_pattern_components(alternative, literals, ranges);
                }
            }

            MatchPattern::Range { .. } => {
                ranges.push(pattern.clone());
            }

            MatchPattern::Wildcard => {}

            _ => {
                literals.push(pattern.clone());
            }
        }
    }

    /// Check whether an expression type can be assigned to a declared type.
    fn check_type_compatibility(expected: &Type, found: &Type) -> SemanticResult<()> {
        if Self::are_compatible(expected, found) {
            Ok(())
        } else {
            Err(SemanticError::TypeMismatch {
                expected: expected.clone(),
                found: found.clone(),
            })
        }
    }

    /// Determine whether two types are compatible.
    fn are_compatible(left: &Type, right: &Type) -> bool {
        left == right
            || (Self::is_numeric(left) && Self::is_numeric(right))
            || *left == Type::Unknown
            || *right == Type::Unknown
    }

    /// Determine whether a type is numeric.
    fn is_numeric(ty: &Type) -> bool {
        matches!(ty, Type::Int | Type::Float)
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}
