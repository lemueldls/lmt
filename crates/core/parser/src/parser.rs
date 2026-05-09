use crate::ast::*;
use crate::lexer::{Lexer, Token};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn expect(&mut self, expected: Token) {
        if self.current_token == expected {
            self.advance();
        } else {
            panic!("Expected {:?}, found {:?}", expected, self.current_token);
        }
    }

    pub fn parse_type(&mut self) -> Type {
        match &self.current_token {
            Token::LBrace => self.parse_refined_type(),
            Token::Ident(s) => {
                let base = match s.as_str() {
                    "Int" => BaseType::Int,
                    "Bool" => BaseType::Bool,
                    "Real" => BaseType::Real,
                    _ => BaseType::Custom(s.clone()),
                };
                self.advance();
                Type::Base(base)
            }
            _ => panic!("Unexpected token for type: {:?}", self.current_token),
        }
    }

    fn parse_refined_type(&mut self) -> Type {
        self.expect(Token::LBrace);
        let v = if let Token::Ident(s) = &self.current_token {
            let name = s.clone();
            self.advance();
            name
        } else {
            panic!("Expected identifier for refinement variable");
        };

        self.expect(Token::Colon);
        let base = match &self.current_token {
            Token::Ident(s) => {
                let bt = match s.as_str() {
                    "Int" => BaseType::Int,
                    "Bool" => BaseType::Bool,
                    "Real" => BaseType::Real,
                    _ => BaseType::Custom(s.clone()),
                };
                self.advance();
                bt
            }
            _ => panic!("Expected base type for refinement"),
        };

        self.expect(Token::Pipe);
        let predicate = self.parse_expr();
        self.expect(Token::RBrace);

        Type::Refined {
            base,
            v,
            predicate,
        }
    }

    pub fn parse_expr(&mut self) -> Expr {
        self.parse_implies()
    }

    fn parse_implies(&mut self) -> Expr {
        let mut left = self.parse_or();
        while self.current_token == Token::Implies {
            self.advance();
            let right = self.parse_or();
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Implies,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();
        while self.current_token == Token::Or {
            self.advance();
            let right = self.parse_and();
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while self.current_token == Token::And {
            self.advance();
            let right = self.parse_equality();
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_relational();
        loop {
            match &self.current_token {
                Token::Eq => {
                    self.advance();
                    let right = self.parse_relational();
                    left = Expr::Binary {
                        left: Box::new(left),
                        op: BinOp::Eq,
                        right: Box::new(right),
                    };
                }
                Token::Ne => {
                    self.advance();
                    let right = self.parse_relational();
                    left = Expr::Binary {
                        left: Box::new(left),
                        op: BinOp::Ne,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        left
    }

    fn parse_relational(&mut self) -> Expr {
        let mut left = self.parse_additive();
        loop {
            let op = match &self.current_token {
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive();
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_multiplicative();
        loop {
            let op = match &self.current_token {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative();
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_multiplicative(&mut self) -> Expr {
        let mut left = self.parse_unary();
        loop {
            let op = match &self.current_token {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary();
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_unary(&mut self) -> Expr {
        match &self.current_token {
            Token::Not => {
                self.advance();
                Expr::Unary {
                    op: UnOp::Not,
                    expr: Box::new(self.parse_unary()),
                }
            }
            Token::Minus => {
                self.advance();
                Expr::Unary {
                    op: UnOp::Neg,
                    expr: Box::new(self.parse_unary()),
                }
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.current_token.clone() {
            Token::Int(n) => {
                self.advance();
                Expr::Literal(Lit::Int(n))
            }
            Token::Real(n) => {
                self.advance();
                Expr::Literal(Lit::Real(n))
            }
            Token::True => {
                self.advance();
                Expr::Literal(Lit::Bool(true))
            }
            Token::False => {
                self.advance();
                Expr::Literal(Lit::Bool(false))
            }
            Token::Ident(s) => {
                self.advance();
                Expr::Var(s)
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr();
                self.expect(Token::RParen);
                expr
            }
            _ => panic!("Unexpected token in primary: {:?}", self.current_token),
        }
    }

    pub fn parse_function_contract(&mut self) -> FunctionContract {
        self.expect(Token::Fn);
        let name = if let Token::Ident(s) = &self.current_token {
            let n = s.clone();
            self.advance();
            n
        } else {
            panic!("Expected function name");
        };

        self.expect(Token::LParen);
        let mut params = Vec::new();
        while self.current_token != Token::RParen {
            let p_name = if let Token::Ident(s) = &self.current_token {
                let n = s.clone();
                self.advance();
                n
            } else {
                panic!("Expected parameter name");
            };
            self.expect(Token::Colon);
            let p_type = self.parse_type();
            params.push((p_name, p_type));

            if self.current_token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(Token::RParen);

        self.expect(Token::Arrow);
        let return_type = self.parse_type();

        let mut pre_conditions = Vec::new();
        let mut post_conditions = Vec::new();

        loop {
            match &self.current_token {
                Token::Pre => {
                    self.advance();
                    pre_conditions.push(self.parse_expr());
                }
                Token::Post => {
                    self.advance();
                    post_conditions.push(self.parse_expr());
                }
                _ => break,
            }
        }

        FunctionContract {
            name,
            params,
            return_type,
            pre_conditions,
            post_conditions,
        }
    }
}

pub fn parse_spec(input: &str) -> Vec<FunctionContract> {
    let mut parser = Parser::new(input);
    let mut contracts = Vec::new();
    while parser.current_token != Token::EOF {
        contracts.push(parser.parse_function_contract());
    }
    contracts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_refined_type() {
        let input = "{v:Int | v > 0}";
        let mut parser = Parser::new(input);
        let ty = parser.parse_type();
        
        match ty {
            Type::Refined { base, v, predicate } => {
                assert_eq!(base, BaseType::Int);
                assert_eq!(v, "v");
                match predicate {
                    Expr::Binary { left, op, right } => {
                        assert_eq!(op, BinOp::Gt);
                        if let Expr::Var(s) = *left {
                            assert_eq!(s, "v");
                        } else { panic!("Expected Var"); }
                        if let Expr::Literal(Lit::Int(n)) = *right {
                            assert_eq!(n, 0);
                        } else { panic!("Expected Literal Int"); }
                    }
                    _ => panic!("Expected Binary Expr"),
                }
            }
            _ => panic!("Expected Refined Type"),
        }
    }

    #[test]
    fn test_parse_function_contract() {
        let input = "fn div(x: Int, y: {v:Int | v != 0}) -> {v:Int | v == x / y} @pre x > 0 @post v >= 0";
        let mut parser = Parser::new(input);
        let contract = parser.parse_function_contract();
        
        assert_eq!(contract.name, "div");
        assert_eq!(contract.params.len(), 2);
        assert_eq!(contract.params[0].0, "x");
        assert_eq!(contract.params[1].0, "y");
        assert_eq!(contract.pre_conditions.len(), 1);
        assert_eq!(contract.post_conditions.len(), 1);
    }
}
