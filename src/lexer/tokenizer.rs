// src/lexer/tokenizer.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Let,
    Print,
    Fn,
    If,
    Else,
    While,
    For,
    In,
    Return,

    // Identifiers
    Identifier(String),

    // Literals
    Integer(i64),
    Float(f64),
    StringLiteral(String),
    InterpolatedString(Vec<StringPart>),
    CharacterLiteral(char),
    True,
    False,
    Null,

    // Operators: +, -, *, /, %, =, ==, !=, <, >, <=, >=, &&, ||, !
    Operator(String),

    // Punctuation: ; : ( ) { } ,
    Symbol(char),

    LeftBracket,
    RightBracket,
    Break,
    Continue,
    Match,
    FatArrow,
    Pipe,
    Range,
    RangeInclusive,

    // Statement boundaries
    Newline,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Expression(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut chars = src.chars();
        let first = chars.next();

        Self {
            chars,
            current: first,
            line: 1,
            column: 1,
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.current = self.chars.next();
    }

    fn make_token(&self, kind: TokenKind, line: usize, column: usize) -> Token {
        Token { kind, line, column }
    }

    fn skip_horizontal_whitespace(&mut self) {
        while matches!(self.current, Some(' ' | '\t' | '\r')) {
            self.advance();
        }
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.current {
            if ch == '\n' {
                break;
            }

            self.advance();
        }
    }

    fn identifier_or_keyword(&mut self, line: usize, column: usize) -> Token {
        let mut identifier = String::new();

        while let Some(ch) = self.current {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match identifier.as_str() {
            "let" => TokenKind::Let,
            "print" => TokenKind::Print,
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,

            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "match" => TokenKind::Match,

            _ => TokenKind::Identifier(identifier),
        };

        self.make_token(kind, line, column)
    }

    fn number(&mut self, line: usize, column: usize) -> Result<Token, String> {
        let mut number = String::new();
        let mut has_decimal = false;

        while let Some(ch) = self.current {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !has_decimal && self.chars.clone().next() != Some('.') {
                has_decimal = true;
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if has_decimal {
            let value = number.parse::<f64>().map_err(|_| {
                format!(
                    "Invalid floating-point literal '{}' at line {}, column {}",
                    number, line, column
                )
            })?;

            if !value.is_finite() {
                return Err(format!(
                    "Floating-point literal out of range at line {}, column {}",
                    line, column
                ));
            }

            Ok(self.make_token(TokenKind::Float(value), line, column))
        } else {
            let value = number.parse::<i64>().map_err(|_| {
                format!(
                    "Integer literal out of range '{}' at line {}, column {}",
                    number, line, column
                )
            })?;

            Ok(self.make_token(TokenKind::Integer(value), line, column))
        }
    }

    fn read_interpolation(&mut self) -> Result<String, String> {
        // Consume opening {
        self.advance();

        let mut expression = String::new();
        let mut brace_depth = 1usize;

        let mut in_string = false;
        let mut in_character = false;
        let mut escaped = false;

        while let Some(ch) = self.current {
            if escaped {
                expression.push(ch);
                escaped = false;
                self.advance();
                continue;
            }

            if (in_string || in_character) && ch == '\\' {
                expression.push(ch);
                escaped = true;
                self.advance();
                continue;
            }

            if in_string {
                expression.push(ch);

                if ch == '"' {
                    in_string = false;
                }

                self.advance();
                continue;
            }

            if in_character {
                expression.push(ch);

                if ch == '\'' {
                    in_character = false;
                }

                self.advance();
                continue;
            }

            match ch {
                '"' => {
                    in_string = true;
                    expression.push(ch);
                }

                '\'' => {
                    in_character = true;
                    expression.push(ch);
                }

                '{' => {
                    brace_depth += 1;
                    expression.push(ch);
                }

                '}' => {
                    brace_depth -= 1;

                    if brace_depth == 0 {
                        self.advance();

                        if expression.trim().is_empty() {
                            return Err(format!(
                                "Empty interpolation at line {}, column {}",
                                self.line, self.column
                            ));
                        }

                        return Ok(expression);
                    }

                    expression.push(ch);
                }

                _ => {
                    // Includes newlines, which are valid inside
                    // multiline interpolation expressions.
                    expression.push(ch);
                }
            }

            self.advance();
        }

        Err(format!(
            "Unterminated interpolation at line {}, column {}",
            self.line, self.column
        ))
    }

    fn string(&mut self, line: usize, column: usize) -> Result<Token, String> {
        self.advance(); // Opening double quote

        let mut text = String::new();
        let mut parts = Vec::new();
        let mut has_interpolation = false;

        loop {
            match self.current {
                Some('"') => {
                    self.advance();
                    break;
                }

                Some('\\') => {
                    self.advance();

                    let escaped = match self.current {
                        Some('n') => '\n',
                        Some('t') => '\t',
                        Some('r') => '\r',
                        Some('0') => '\0',
                        Some('"') => '"',
                        Some('\\') => '\\',
                        Some('{') => '{',
                        Some('}') => '}',

                        Some(other) => {
                            return Err(format!(
                                "Invalid string escape '\\{}' at line {}, column {}",
                                other, self.line, self.column
                            ));
                        }

                        None => {
                            return Err(format!(
                                "Unterminated escape sequence at line {}, column {}",
                                self.line, self.column
                            ));
                        }
                    };

                    text.push(escaped);
                    self.advance();
                }

                Some('{') => {
                    has_interpolation = true;

                    if !text.is_empty() {
                        parts.push(StringPart::Text(std::mem::take(&mut text)));
                    }

                    let expression = self.read_interpolation()?;

                    parts.push(StringPart::Expression(expression));
                }

                Some('\n') | None => {
                    return Err(format!(
                        "Unterminated string literal at line {}, column {}",
                        line, column
                    ));
                }

                Some(ch) => {
                    text.push(ch);
                    self.advance();
                }
            }
        }

        if has_interpolation {
            if !text.is_empty() {
                parts.push(StringPart::Text(text));
            }

            Ok(self.make_token(TokenKind::InterpolatedString(parts), line, column))
        } else {
            Ok(self.make_token(TokenKind::StringLiteral(text), line, column))
        }
    }

    fn character(&mut self, line: usize, column: usize) -> Result<Token, String> {
        self.advance(); // Opening single quote

        let value = match self.current {
            Some('\\') => {
                self.advance();

                let escaped = match self.current {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('0') => '\0',
                    Some('\'') => '\'',
                    Some('\\') => '\\',

                    Some(other) => {
                        return Err(format!(
                            "Invalid character escape '\\{}' at line {}, column {}",
                            other, line, column
                        ));
                    }

                    None => {
                        return Err(format!(
                            "Unterminated character literal at line {}, column {}",
                            line, column
                        ));
                    }
                };

                self.advance();
                escaped
            }

            Some(ch) if ch != '\'' && ch != '\n' => {
                self.advance();
                ch
            }

            _ => {
                return Err(format!(
                    "Invalid character literal at line {}, column {}",
                    line, column
                ));
            }
        };

        if self.current != Some('\'') {
            return Err(format!(
                "Character literal must contain exactly one Unicode scalar value at line {}, column {}",
                line, column
            ));
        }

        self.advance(); // Closing single quote

        Ok(self.make_token(TokenKind::CharacterLiteral(value), line, column))
    }

    fn range_operator(&mut self, line: usize, column: usize) -> Result<Token, String> {
        self.advance(); // First dot

        if self.current != Some('.') {
            return Err(format!(
                "Unexpected '.' at line {}, column {}",
                line, column
            ));
        }

        self.advance(); // Second dot

        if self.current == Some('=') {
            self.advance();

            Ok(self.make_token(TokenKind::RangeInclusive, line, column))
        } else {
            Ok(self.make_token(TokenKind::Range, line, column))
        }
    }

    fn operator(&mut self, line: usize, column: usize) -> Result<Token, String> {
        let first = self.current.unwrap();
        self.advance();

        // Match-arm operator: =>
        if first == '=' && self.current == Some('>') {
            self.advance();

            return Ok(self.make_token(TokenKind::FatArrow, line, column));
        }

        // A single | separates OR-pattern alternatives.
        // A double || remains logical OR.
        if first == '|' && self.current != Some('|') {
            return Ok(self.make_token(TokenKind::Pipe, line, column));
        }

        let mut operator = String::from(first);

        let supports_equals = matches!(first, '=' | '!' | '<' | '>');
        let supports_double = matches!(first, '&' | '|');

        if supports_equals && self.current == Some('=') {
            operator.push('=');
            self.advance();
        } else if supports_double && self.current == Some(first) {
            operator.push(first);
            self.advance();
        } else if supports_double {
            return Err(format!(
                "Invalid operator '{}' at line {}, column {}",
                operator, line, column
            ));
        }

        Ok(self.make_token(TokenKind::Operator(operator), line, column))
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        loop {
            self.skip_horizontal_whitespace();

            // Comments preserve the following newline token.
            if self.current == Some('#') {
                self.skip_comment();
                continue;
            }

            break;
        }

        let start_line = self.line;
        let start_column = self.column;

        match self.current {
            None => Ok(self.make_token(TokenKind::Eof, start_line, start_column)),

            Some('\n') => {
                self.advance();

                Ok(self.make_token(TokenKind::Newline, start_line, start_column))
            }

            Some('"') => self.string(start_line, start_column),

            Some('\'') => self.character(start_line, start_column),

            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                Ok(self.identifier_or_keyword(start_line, start_column))
            }

            Some(ch) if ch.is_ascii_digit() => self.number(start_line, start_column),

            Some(ch)
                if matches!(
                    ch,
                    '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|'
                ) =>
            {
                self.operator(start_line, start_column)
            }

            Some('[') => {
                self.advance();

                Ok(self.make_token(TokenKind::LeftBracket, start_line, start_column))
            }

            Some(']') => {
                self.advance();

                Ok(self.make_token(TokenKind::RightBracket, start_line, start_column))
            }

            Some(ch) if matches!(ch, ';' | ':' | '(' | ')' | '{' | '}' | ',') => {
                self.advance();

                Ok(self.make_token(TokenKind::Symbol(ch), start_line, start_column))
            }

            Some('.') => self.range_operator(start_line, start_column),

            Some(ch) => Err(format!(
                "Unexpected character '{}' at line {}, column {}",
                ch, start_line, start_column
            )),
        }
    }

    pub fn tokenize(src: &'a str) -> Result<Vec<Token>, String> {
        let mut lexer = Lexer::new(src);
        let mut tokens = Vec::new();

        loop {
            let token = lexer.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);

            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}
