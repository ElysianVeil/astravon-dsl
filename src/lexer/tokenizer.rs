// src/lexer/tokenizer.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Integer(i64),
    Float(f64),
    StringLiteral(String),
    CharacterLiteral(char),
    Keyword(String),
    Symbol(char),
    Eof,
}

#[derive(Debug, Clone)]
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
            column: 0,
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current {
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }

        self.current = self.chars.next();
    }

    fn make_token(&self, kind: TokenKind, line: usize, column: usize) -> Token {
        Token { kind, line, column }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        // Skip whitespace.
        while let Some(ch) = self.current {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }

        let start_line = self.line;
        let start_col = self.column;

        match self.current {
            // String literals.
            Some('"') => {
                self.advance();

                let mut value = String::new();

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

                                Some(other) => {
                                    return Err(format!(
                                        "Invalid string escape '\\{}' at line {}",
                                        other, self.line
                                    ));
                                }

                                None => {
                                    return Err(format!(
                                        "Unterminated escape sequence at line {}",
                                        self.line
                                    ));
                                }
                            };

                            value.push(escaped);
                            self.advance();
                        }

                        Some('\n') | None => {
                            return Err(format!(
                                "Unterminated string literal at line {}, column {}",
                                start_line, start_col
                            ));
                        }

                        Some(ch) => {
                            value.push(ch);
                            self.advance();
                        }
                    }
                }

                Ok(self.make_token(TokenKind::StringLiteral(value), start_line, start_col))
            }

            // Identifiers and keywords.
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                let mut ident = String::new();

                while let Some(c) = self.current {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                let keywords = ["print", "true", "false", "null"];

                let kind = if keywords.contains(&ident.as_str()) {
                    TokenKind::Keyword(ident)
                } else {
                    TokenKind::Identifier(ident)
                };

                Ok(self.make_token(kind, start_line, start_col))
            }

            // Numeric literals.
            Some(ch) if ch.is_ascii_digit() => {
                let mut number = String::new();
                let mut has_decimal = false;

                while let Some(c) = self.current {
                    if c.is_ascii_digit() {
                        number.push(c);
                        self.advance();
                    } else if c == '.' && !has_decimal {
                        has_decimal = true;
                        number.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                if has_decimal {
                    let value = number.parse::<f64>().map_err(|_| {
                        format!(
                            "Invalid floating-point literal '{}' at line {}",
                            number, start_line
                        )
                    })?;

                    if !value.is_finite() {
                        return Err(format!(
                            "Floating-point literal out of range at line {}",
                            start_line
                        ));
                    }

                    Ok(self.make_token(TokenKind::Float(value), start_line, start_col))
                } else {
                    let value = number.parse::<i64>().map_err(|_| {
                        format!(
                            "Integer literal out of range '{}' at line {}",
                            number, start_line
                        )
                    })?;

                    Ok(self.make_token(TokenKind::Integer(value), start_line, start_col))
                }
            }

            // Character literals.
            Some('\'') => {
                self.advance();

                let character = match self.current {
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
                                    "Invalid character escape '\\{}' at line {}",
                                    other, start_line
                                ));
                            }

                            None => {
                                return Err(format!(
                                    "Unterminated character literal at line {}",
                                    start_line
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
                            start_line, start_col
                        ));
                    }
                };

                if self.current != Some('\'') {
                    return Err(format!(
                        "Character literal must contain exactly one character at line {}",
                        start_line
                    ));
                }

                self.advance();

                Ok(self.make_token(
                    TokenKind::CharacterLiteral(character),
                    start_line,
                    start_col,
                ))
            }

            // Symbols.
            Some(ch) => {
                self.advance();

                Ok(self.make_token(TokenKind::Symbol(ch), start_line, start_col))
            }

            // End of input.
            None => Ok(self.make_token(TokenKind::Eof, start_line, start_col)),
        }
    }

    /// Tokenize the entire source.
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
