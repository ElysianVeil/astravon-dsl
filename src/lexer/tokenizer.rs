// src/lexer/tokenizer.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Number(f64),
    StringLiteral(String),
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
    src: &'a str,
    chars: std::str::Chars<'a>,
    current: Option<char>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut chars = src.chars();
        let first = chars.next();
        Lexer {
            src,
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

    pub fn next_token(&mut self) -> Token {
        // skip whitespace
        while let Some(ch) = self.current {
            if ch.is_whitespace() {
                self.advance();
                continue;
            }
            break;
        }

        match self.current {
            Some('"') => {
                // Start of a string literal
                let start_col = self.column;
                self.advance(); // skip opening quote
                let mut value = String::new();
                while let Some(c) = self.current {
                    if c == '"' {
                        self.advance(); // skip closing quote
                        break;
                    }
                    value.push(c);
                    self.advance();
                }
                Token {
                    kind: TokenKind::StringLiteral(value),
                    line: self.line,
                    column: start_col,
                }
            }
            Some(ch) if ch.is_ascii_alphabetic() => {
                let start_col = self.column;
                let mut ident = String::new();
                while let Some(c) = self.current {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                // Keyword detection
                let keywords = ["print"]; // Add more DSL keywords here
                let kind = if keywords.contains(&ident.as_str()) {
                    TokenKind::Keyword(ident.clone())
                } else {
                    TokenKind::Identifier(ident.clone())
                };
                // you can add keyword detection here
                Token {
                    kind,
                    line: self.line,
                    column: start_col,
                }
            }
            Some(ch) if ch.is_ascii_digit() => {
                let start_col = self.column;
                let mut num = String::new();
                while let Some(c) = self.current {
                    if c.is_ascii_digit() || c == '.' {
                        num.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                Token {
                    kind: TokenKind::Number(num.parse::<f64>().unwrap_or(0.0)),
                    line: self.line,
                    column: start_col,
                }
            }
            Some(ch) => {
                let tok = Token {
                    kind: TokenKind::Symbol(ch),
                    line: self.line,
                    column: self.column,
                };
                self.advance();
                tok
            }
            None => Token {
                kind: TokenKind::Eof,
                line: self.line,
                column: self.column,
            },
        }
    }

    /// Tokenize the entire source into a vector of Tokens
    pub fn tokenize(src: &'a str) -> Result<Vec<Token>, String> {
        let mut lexer = Lexer::new(src);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            let is_eof = matches!(tok.kind, TokenKind::Eof);
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        for token in &tokens {
            println!("Got token: {:?}", token);
        }
        Ok(tokens)
    }
}
