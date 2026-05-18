use lmt_diagnostics::ModuleId;

use crate::{
    diagnostic::Diagnostic,
    token::{Token, TokenError, TokenKind},
};

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    module_id: ModuleId,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, module_id: ModuleId) -> Self {
        Self {
            input,
            pos: 0,
            module_id,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_trivia();

        if self.pos >= self.input.len() {
            return Token::new(TokenKind::EOF, self.pos, self.pos, self.module_id);
        }

        let start = self.pos;
        let c = self.input.as_bytes()[self.pos] as char;

        match c {
            '(' => self.single(TokenKind::LParen),
            ')' => self.single(TokenKind::RParen),
            '{' => self.single(TokenKind::LBrace),
            '}' => self.single(TokenKind::RBrace),
            '[' => self.single(TokenKind::LBracket),
            ']' => self.single(TokenKind::RBracket),
            ',' => self.single(TokenKind::Comma),
            ';' => self.single(TokenKind::Semi),
            '.' => self.single(TokenKind::Dot),
            '|' => self.single(TokenKind::Pipe),
            '&' => self.single(TokenKind::Amp),
            '^' => self.single(TokenKind::Caret),
            '%' => self.single(TokenKind::Percent),
            '+' => self.single(TokenKind::Plus),
            '*' => self.single(TokenKind::Star),
            '/' => self.single(TokenKind::Slash),
            '?' => {
                self.pos += 1;
                if self.match_char('?') {
                    Token::new(TokenKind::DoubleHole, start, self.pos, self.module_id)
                } else {
                    Token::new(TokenKind::Hole, start, self.pos, self.module_id)
                }
            }
            '@' => self.lex_directive(),
            '"' => self.lex_string(),
            ':' => {
                self.pos += 1;
                if self.match_char(':') {
                    Token::new(TokenKind::ColonColon, start, self.pos, self.module_id)
                } else {
                    Token::new(TokenKind::Colon, start, self.pos, self.module_id)
                }
            }
            '=' => {
                self.pos += 1;
                if self.match_char('=') {
                    Token::new(TokenKind::EqEq, start, self.pos, self.module_id)
                } else if self.match_char('>') {
                    Token::new(TokenKind::FatArrow, start, self.pos, self.module_id)
                } else {
                    Token::new(TokenKind::Assign, start, self.pos, self.module_id)
                }
            }
            '!' => {
                self.pos += 1;
                if self.match_char('=') {
                    Token::new(TokenKind::Ne, start, self.pos, self.module_id)
                } else {
                    Token::new(
                        // TokenKind::Error("unexpected '!'".to_string()),
                        TokenKind::Error(TokenError::UnexpectedToken(Box::new(TokenKind::Not))),
                        start,
                        self.pos,
                        self.module_id,
                    )
                }
            }
            '<' => {
                self.pos += 1;
                if self.match_char('=') {
                    Token::new(TokenKind::Le, start, self.pos, self.module_id)
                } else {
                    Token::new(TokenKind::Lt, start, self.pos, self.module_id)
                }
            }
            '>' => {
                self.pos += 1;
                if self.match_char('=') {
                    Token::new(TokenKind::Ge, start, self.pos, self.module_id)
                } else {
                    Token::new(TokenKind::Gt, start, self.pos, self.module_id)
                }
            }
            '-' => {
                self.pos += 1;
                if self.match_char('>') {
                    Token::new(TokenKind::Arrow, start, self.pos, self.module_id)
                } else {
                    Token::new(TokenKind::Minus, start, self.pos, self.module_id)
                }
            }
            c if c.is_ascii_digit() => self.lex_number(),
            c if is_ident_start(c) => self.lex_identifier(),
            _ => {
                self.pos += 1;
                Token::new(
                    TokenKind::Error(TokenError::UnknownToken),
                    start,
                    self.pos,
                    self.module_id,
                )
            }
        }
    }

    fn single(&mut self, kind: TokenKind) -> Token {
        let start = self.pos;
        self.pos += 1;
        Token::new(kind, start, self.pos, self.module_id)
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] as char == expected {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn skip_trivia(&mut self) {
        loop {
            while let Some(b) = self.input.as_bytes().get(self.pos) {
                if (*b as char).is_whitespace() {
                    self.pos += 1;
                } else {
                    break;
                }
            }

            if self.starts_with("//") {
                self.pos += 2;
                while self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }

            if self.starts_with("/*") {
                self.pos += 2;
                while self.pos + 1 < self.input.len() && !self.starts_with("*/") {
                    self.pos += 1;
                }
                if self.starts_with("*/") {
                    self.pos += 2;
                }
                continue;
            }

            break;
        }
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.input[self.pos..].starts_with(prefix)
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.pos;
        self.pos += 1;
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos] as char;
            if is_ident_continue(c) {
                self.pos += 1;
            } else {
                break;
            }
        }

        let text = &self.input[start..self.pos];
        let kind = match text {
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "use" => TokenKind::Use,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Ident(text.to_string()),
        };

        Token::new(kind, start, self.pos, self.module_id)
    }

    fn lex_directive(&mut self) -> Token {
        let start = self.pos;
        self.pos += 1;

        if self.pos >= self.input.len() || !is_ident_start(self.input.as_bytes()[self.pos] as char)
        {
            return Token::new(
                TokenKind::Error(TokenError::InvalidDirective),
                start,
                self.pos,
                self.module_id,
            );
        }

        self.pos += 1;
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos] as char;
            if is_ident_continue(c) {
                self.pos += 1;
            } else {
                break;
            }
        }

        Token::new(
            TokenKind::Directive(self.input[start + 1..self.pos].to_string()),
            start,
            self.pos,
            self.module_id,
        )
    }

    fn lex_string(&mut self) -> Token {
        let start = self.pos;
        self.pos += 1; // opening quote

        let mut value = String::new();
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos] as char;
            self.pos += 1;
            match c {
                '"' => {
                    return Token::new(TokenKind::String(value), start, self.pos, self.module_id);
                }
                '\\' => {
                    if self.pos >= self.input.len() {
                        break;
                    }
                    let escaped = self.input.as_bytes()[self.pos] as char;
                    self.pos += 1;
                    value.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '"' => '"',
                        '\\' => '\\',
                        other => other,
                    });
                }
                other => value.push(other),
            }
        }

        Token::new(
            TokenKind::Error(TokenError::UnterminatedString),
            start,
            self.pos,
            self.module_id,
        )
    }

    fn lex_number(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        let is_real = self.pos + 1 < self.input.len()
            && self.input.as_bytes()[self.pos] as char == '.'
            && self.input.as_bytes()[self.pos + 1].is_ascii_digit();

        if is_real {
            self.pos += 1;
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            let text = &self.input[start..self.pos];
            return match text.parse::<f64>() {
                Ok(value) => Token::new(TokenKind::Real(value), start, self.pos, self.module_id),
                Err(_) => {
                    Token::new(
                        TokenKind::Error(TokenError::InvalidReal),
                        start,
                        self.pos,
                        self.module_id,
                    )
                }
            };
        }

        let text = &self.input[start..self.pos];
        match text.parse::<i64>() {
            Ok(value) => Token::new(TokenKind::Int(value), start, self.pos, self.module_id),
            Err(_) => {
                Token::new(
                    TokenKind::Error(TokenError::InvalidInteger),
                    start,
                    self.pos,
                    self.module_id,
                )
            }
        }
    }
}

pub fn tokenize_with_diagnostics(
    input: &str,
    module_id: ModuleId,
) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(input, module_id);
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();

    loop {
        let token = lexer.next_token();
        if let TokenKind::Error(error) = &token.kind {
            diagnostics.push(error.diagnostic(token.span));
        }

        let is_eof = matches!(token.kind, TokenKind::EOF);
        tokens.push(token);
        if is_eof {
            break;
        }
    }

    (tokens, diagnostics)
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
