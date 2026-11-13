// src/ast/nodes.rs

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        type_annotation: Option<TypeAnnotation>,
        initializer: Expression,
    },

    Assignment {
        name: String,
        value: Expression,
    },

    Print(Expression),

    Function {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: Vec<Statement>,
    },

    Return(Option<Expression>),

    Expression(Expression),

    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },

    While {
        condition: Expression,
        body: Vec<Statement>,
    },

    For {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },

    Break,

    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    StringLiteral(String),
    InterpolatedString(Vec<StringSegment>),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    BooleanLiteral(bool),
    CharacterLiteral(char),
    NullLiteral,

    Identifier(String),

    Call {
        name: String,
        arguments: Vec<Expression>,
    },

    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },

    Array(Vec<Expression>),

    Match {
        value: Box<Expression>,
        arms: Vec<MatchArm>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringSegment {
    Text(String),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,

    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    Int,
    Float,
    Bool,
    Char,
    String,
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub guard: Option<Expression>,
    pub expression: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),

    Wildcard,

    Or(Vec<MatchPattern>),

    Range {
        start: Box<MatchPattern>,
        end: Box<MatchPattern>,
        inclusive: bool,
    },
}
