use std::{iter::Peekable, str::Chars};

use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[repr(u8)]
pub enum Token {
    // Keywords
    Fn,
    Type,
    True,
    False,

    // Braces and Punctuation
    LBrace, // {
    RBrace, // }
    LParen, // (
    RParen, // )
    Pipe,   // |
    Colon,  // :
    Comma,  // ,
    Arrow,  // ->

    // Annotations
    Pre,    // @pre
    Post,   // @post
    Assert, // @assert

    // Operators
    And,     // &&
    Or,      // ||
    Not,     // !
    Implies, // =>

    Eq,     // ==
    Assign, // =
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Gt,     // >
    Ge,     // >=

    Plus,  // +
    Minus, // -
    Star,  // *
    Slash, // /

    // Literals and Identifiers
    Ident(String),
    Int(i64),
    Real(f64),

    EOF,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.input.next() {
            Some('{') => Token::LBrace,
            Some('}') => Token::RBrace,
            Some('(') => Token::LParen,
            Some(')') => Token::RParen,
            Some('|') => {
                if let Some('|') = self.input.peek() {
                    self.input.next();
                    Token::Or
                } else {
                    Token::Pipe
                }
            }
            Some(':') => Token::Colon,
            Some(',') => Token::Comma,
            Some('-') => {
                if let Some('>') = self.input.peek() {
                    self.input.next();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            Some('@') => self.lex_annotation(),
            Some('&') => {
                if let Some('&') = self.input.peek() {
                    self.input.next();
                    Token::And
                } else {
                    panic!("Expected &&")
                }
            }
            Some('!') => {
                if let Some('=') = self.input.peek() {
                    self.input.next();
                    Token::Ne
                } else {
                    Token::Not
                }
            }
            Some('=') => {
                if let Some('=') = self.input.peek() {
                    self.input.next();
                    Token::Eq
                } else if let Some('>') = self.input.peek() {
                    self.input.next();
                    Token::Implies
                } else {
                    Token::Assign
                }
            }
            Some('<') => {
                if let Some('=') = self.input.peek() {
                    self.input.next();
                    Token::Le
                } else {
                    Token::Lt
                }
            }
            Some('>') => {
                if let Some('=') = self.input.peek() {
                    self.input.next();
                    Token::Ge
                } else {
                    Token::Gt
                }
            }
            Some('+') => Token::Plus,
            Some('*') => Token::Star,
            Some('/') => Token::Slash,
            Some(c) if c.is_alphabetic() || c == '_' => self.lex_ident(c),
            Some(c) if c.is_numeric() => self.lex_number(c),
            None => Token::EOF,
            Some(c) => panic!("Unexpected character: {}", c),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.input.peek() {
            if c.is_whitespace() {
                self.input.next();
            } else {
                break;
            }
        }
    }

    fn lex_ident(&mut self, first: char) -> Token {
        let mut ident = first.to_string();
        while let Some(&c) = self.input.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(self.input.next().unwrap());
            } else {
                break;
            }
        }

        match ident.as_str() {
            "fn" => Token::Fn,
            "type" => Token::Type,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Ident(ident),
        }
    }

    fn lex_number(&mut self, first: char) -> Token {
        let mut s = first.to_string();
        let mut is_real = false;
        while let Some(&c) = self.input.peek() {
            if c.is_numeric() {
                s.push(self.input.next().unwrap());
            } else if c == '.' {
                is_real = true;
                s.push(self.input.next().unwrap());
            } else {
                break;
            }
        }

        if is_real {
            Token::Real(s.parse().unwrap())
        } else {
            Token::Int(s.parse().unwrap())
        }
    }

    fn lex_annotation(&mut self) -> Token {
        let mut s = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_alphabetic() {
                s.push(self.input.next().unwrap());
            } else {
                break;
            }
        }

        match s.as_str() {
            "pre" => Token::Pre,
            "post" => Token::Post,
            "assert" => Token::Assert,
            _ => panic!("Unknown annotation: @{}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let input = "fn type { } | : , -> @pre @post @assert && || ! => == = != < <= > >= + - * / ident 123 45.6 true false";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::Fn);
        assert_eq!(lexer.next_token(), Token::Type);
        assert_eq!(lexer.next_token(), Token::LBrace);
        assert_eq!(lexer.next_token(), Token::RBrace);
        assert_eq!(lexer.next_token(), Token::Pipe);
        assert_eq!(lexer.next_token(), Token::Colon);
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Arrow);
        assert_eq!(lexer.next_token(), Token::Pre);
        assert_eq!(lexer.next_token(), Token::Post);
        assert_eq!(lexer.next_token(), Token::Assert);
        assert_eq!(lexer.next_token(), Token::And);
        assert_eq!(lexer.next_token(), Token::Or);
        assert_eq!(lexer.next_token(), Token::Not);
        assert_eq!(lexer.next_token(), Token::Implies);
        assert_eq!(lexer.next_token(), Token::Eq);
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Ne);
        assert_eq!(lexer.next_token(), Token::Lt);
        assert_eq!(lexer.next_token(), Token::Le);
        assert_eq!(lexer.next_token(), Token::Gt);
        assert_eq!(lexer.next_token(), Token::Ge);
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Star);
        assert_eq!(lexer.next_token(), Token::Slash);
        assert_eq!(lexer.next_token(), Token::Ident("ident".to_string()));
        assert_eq!(lexer.next_token(), Token::Int(123));
        assert_eq!(lexer.next_token(), Token::Real(45.6));
        assert_eq!(lexer.next_token(), Token::True);
        assert_eq!(lexer.next_token(), Token::False);
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_lexer_refinement() {
        let input = "{ v: Int | v > 0 }";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::LBrace);
        assert_eq!(lexer.next_token(), Token::Ident("v".to_string()));
        assert_eq!(lexer.next_token(), Token::Colon);
        assert_eq!(lexer.next_token(), Token::Ident("Int".to_string()));
        assert_eq!(lexer.next_token(), Token::Pipe);
        assert_eq!(lexer.next_token(), Token::Ident("v".to_string()));
        assert_eq!(lexer.next_token(), Token::Gt);
        assert_eq!(lexer.next_token(), Token::Int(0));
        assert_eq!(lexer.next_token(), Token::RBrace);
    }
}
