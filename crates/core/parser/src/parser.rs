use crate::{
    ast::*,
    lexer::{Lexer, Token},
};

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
        if self.current_token == Token::LParen {
            self.advance();
            let name = if let Token::Ident(s) = &self.current_token {
                s.clone()
            } else {
                panic!("Expected identifier in named return type");
            };
            self.advance();
            self.expect(Token::Colon);
            let base = match &self.current_token {
                Token::Ident(s) => {
                    let bt = Self::parse_base_type_name(s);
                    self.advance();
                    bt
                }
                _ => panic!("Expected base type"),
            };
            let predicate = if self.current_token == Token::Pipe {
                self.advance();
                self.parse_expr()
            } else {
                Expr::Literal(Lit::Bool(true))
            };
            self.expect(Token::RParen);
            return Type::Refined {
                base,
                v: name,
                predicate,
            };
        }

        let base = match &self.current_token {
            Token::Ident(s) => {
                let bt = Self::parse_base_type_name(s);
                self.advance();
                bt
            }
            _ => panic!("Unexpected token for type: {:?}", self.current_token),
        };

        if self.current_token == Token::Pipe {
            self.advance();
            let predicate = self.parse_expr();
            Type::Refined {
                base,
                v: "it".to_string(),
                predicate,
            }
        } else {
            Type::Base(base)
        }
    }

    fn parse_base_type_name(s: &str) -> BaseType {
        match s {
            "Int" => BaseType::Int,
            "Bool" => BaseType::Bool,
            "Real" => BaseType::Real,
            _ => BaseType::Custom(s.to_string()),
        }
    }

    fn consume_optional_colon(&mut self) {
        if self.current_token == Token::Colon {
            self.advance();
        }
    }

    pub fn parse_expr(&mut self) -> Expr {
        if self.current_token == Token::Let {
            self.advance();
            let name = if let Token::Ident(s) = &self.current_token {
                let n = s.clone();
                self.advance();
                n
            } else {
                panic!("Expected identifier after let");
            };

            let ty = if self.current_token == Token::Colon {
                self.advance();
                Some(Box::new(self.parse_type()))
            } else {
                None
            };

            self.expect(Token::Assign);
            let value = Box::new(self.parse_expr());

            if self.current_token == Token::Semi {
                self.advance();
            }

            let body = Box::new(self.parse_expr());

            return Expr::Let {
                name,
                ty,
                value,
                body,
            };
        }

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

    pub fn parse_function_contract(&mut self, name: String) -> FunctionContract {
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

        self.expect(Token::Colon);
        let return_type = self.parse_type();

        FunctionContract {
            name,
            params,
            return_type,
        }
    }

    pub fn parse_type_alias(&mut self, name: String) -> TypeAlias {
        self.expect(Token::Assign);
        let ty = self.parse_type();
        TypeAlias { name, ty }
    }

    pub fn parse_assertion(&mut self) -> Assertion {
        self.expect(Token::Assert);
        self.consume_optional_colon();
        Assertion {
            predicate: self.parse_expr(),
        }
    }

    pub fn parse_spec_item(&mut self) -> SpecItem {
        match self.current_token {
            Token::Let => {
                self.advance();
                let name = if let Token::Ident(s) = &self.current_token {
                    let n = s.clone();
                    self.advance();
                    n
                } else {
                    panic!("Expected identifier after let");
                };

                if self.current_token == Token::LParen {
                    SpecItem::FunctionContract(self.parse_function_contract(name))
                } else if self.current_token == Token::Assign {
                    SpecItem::TypeAlias(self.parse_type_alias(name))
                } else {
                    panic!("Expected '(' or '=' after let ident");
                }
            }
            Token::Assert => SpecItem::Assertion(self.parse_assertion()),
            _ => panic!("Unexpected token at top level: {:?}", self.current_token),
        }
    }
}

pub fn parse_spec(input: &str) -> Vec<FunctionContract> {
    let mut parser = Parser::new(input);
    let mut contracts = Vec::new();
    while parser.current_token != Token::EOF {
        match parser.parse_spec_item() {
            SpecItem::FunctionContract(contract) => contracts.push(contract),
            SpecItem::TypeAlias(_) | SpecItem::Assertion(_) => {}
        }
    }
    contracts
}

pub fn parse_spec_items(input: &str) -> Vec<SpecItem> {
    let mut parser = Parser::new(input);
    let mut items = Vec::new();
    while parser.current_token != Token::EOF {
        items.push(parser.parse_spec_item());
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_refined_type() {
        let input = "Int | it > 0";
        let mut parser = Parser::new(input);
        let ty = parser.parse_type();

        match ty {
            Type::Refined { base, v, predicate } => {
                assert_eq!(base, BaseType::Int);
                assert_eq!(v, "it");
                match predicate {
                    Expr::Binary { left, op, right } => {
                        assert_eq!(op, BinOp::Gt);
                        if let Expr::Var(s) = *left {
                            assert_eq!(s, "it");
                        } else {
                            panic!("Expected Var");
                        }
                        if let Expr::Literal(Lit::Int(n)) = *right {
                            assert_eq!(n, 0);
                        } else {
                            panic!("Expected Literal Int");
                        }
                    }
                    _ => panic!("Expected Binary Expr"),
                }
            }
            _ => panic!("Expected Refined Type"),
        }
    }

    #[test]
    fn test_parse_function_contract() {
        let input = "let div(x: Int, y: Int | y != 0): (res: Int | res * y == x)";
        let mut parser = Parser::new(input);
        let item = parser.parse_spec_item();
        let contract = match item {
            SpecItem::FunctionContract(c) => c,
            _ => panic!("Expected FunctionContract"),
        };

        assert_eq!(contract.name, "div");
        assert_eq!(contract.params.len(), 2);
        assert_eq!(contract.params[0].0, "x");
        assert_eq!(contract.params[1].0, "y");
        match &contract.return_type {
            Type::Refined {
                base,
                v,
                predicate: _,
            } => {
                assert_eq!(base, &BaseType::Int);
                assert_eq!(v, "res");
            }
            _ => panic!("Expected named refined return type"),
        }
    }

    #[test]
    fn test_parse_type_alias() {
        let input = "let Nat = Int | it >= 0";
        let mut parser = Parser::new(input);
        let item = parser.parse_spec_item();
        let alias = match item {
            SpecItem::TypeAlias(a) => a,
            _ => panic!("Expected TypeAlias"),
        };

        assert_eq!(alias.name, "Nat");
        match alias.ty {
            Type::Refined { .. } => {}
            _ => panic!("expected refined type alias"),
        }
    }

    #[test]
    fn test_parse_spec_items_mixed() {
        let input = "let Nat = Int | it >= 0 let inc(x: Nat): (res: Nat | res == x + 1) @assert: 1 + 1 == 2";
        let items = parse_spec_items(input);

        assert_eq!(items.len(), 3);
        match &items[0] {
            SpecItem::TypeAlias(alias) => assert_eq!(alias.name, "Nat"),
            _ => panic!("expected type alias"),
        }
        match &items[1] {
            SpecItem::FunctionContract(contract) => assert_eq!(contract.name, "inc"),
            _ => panic!("expected function contract"),
        }
        match &items[2] {
            SpecItem::Assertion(_) => {}
            _ => panic!("expected assertion"),
        }
    }
}
