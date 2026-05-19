use lmt_diagnostics::{ModuleId, Span, source::NamedSource};
use picante::PicanteResult;

use crate::{
    ast::{self, Expr, LetDecl, MatchArm, Pattern, Program, Statement, UseDecl},
    db::{SyntaxDatabaseTrait, tokenize},
    diagnostic::Diagnostic,
    lexer::tokenize_with_diagnostics,
    token::{Token, TokenKind},
};

#[derive(Debug)]
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
    module_id: ModuleId,
}

impl Parser {
    fn new(tokens: Vec<Token>, module_id: ModuleId) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics: Vec::new(),
            module_id,
        }
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if !matches!(t.kind, TokenKind::EOF) {
            self.pos = (self.pos + 1).min(self.tokens.len() - 1);
        }
        t
    }

    fn consume_if(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self) -> Option<String> {
        match self.peek_kind() {
            TokenKind::Ident(s) => {
                let name = s.clone();
                self.bump();
                Some(name)
            }
            _ => None,
        }
    }

    fn mark_span_from(&self, start_idx: usize, end_idx: usize) -> Span {
        let start = self.tokens[start_idx].span.start().unwrap();
        let end = self.tokens[end_idx].span.end().unwrap();

        Span::new(start, end, self.module_id)
    }

    fn current_token_span(&self) -> Span {
        if self.tokens.is_empty() {
            return Span::new(0, 0, self.module_id);
        }

        let idx = self.pos.min(self.tokens.len().saturating_sub(1));

        self.tokens[idx].span.clone()
    }

    fn parse_program(&mut self) -> Vec<(Statement, Span)> {
        let mut statements = Vec::new();

        while !matches!(self.peek_kind(), TokenKind::EOF) {
            if let Some((stmt, span)) = self.parse_statement() {
                statements.push((stmt, span));
            } else {
                // Recovery: skip a token to make progress
                if !matches!(self.peek_kind(), TokenKind::EOF) {
                    self.bump();
                }
            }
        }

        statements
    }

    fn parse_statement(&mut self) -> Option<(Statement, Span)> {
        let start_idx = self.pos;
        let result = match self.peek_kind() {
            TokenKind::Let => {
                self.bump();
                let name = self.expect_ident().unwrap_or_else(|| "<anon>".to_string());
                let mut annotation = None;
                let mut value = None;

                if self.consume_if(&TokenKind::Colon) {
                    annotation = Some(self.parse_verification_expr());
                }

                if self.consume_if(&TokenKind::Assign) {
                    value = Some(self.parse_verification_expr());
                }

                // optional semicolon
                if matches!(self.peek_kind(), TokenKind::Semi) {
                    self.bump();
                }

                Some(Statement::Let(LetDecl {
                    name,
                    annotation,
                    value,
                }))
            }
            TokenKind::Use => {
                self.bump();
                let mut path = Vec::new();
                if let Some(first) = self.expect_ident() {
                    path.push(first);
                    while self.consume_if(&TokenKind::Dot) {
                        if let Some(next) = self.expect_ident() {
                            path.push(next);
                        } else {
                            break;
                        }
                    }
                }
                if matches!(self.peek_kind(), TokenKind::Semi) {
                    self.bump();
                }
                Some(Statement::Use(UseDecl { path }))
            }
            _ => {
                // ExprStmt
                let expr = self.parse_verification_expr();
                // expect semicolon
                if matches!(self.peek_kind(), TokenKind::Semi) {
                    self.bump();
                }

                Some(Statement::Expr(expr))
            }
        };

        if let Some(stmt) = result {
            let end_idx = if self.pos == 0 {
                0
            } else {
                self.pos.saturating_sub(1)
            };
            let span = self.mark_span_from(start_idx, end_idx);

            Some((stmt, span))
        } else {
            None
        }
    }

    fn parse_verification_expr(&mut self) -> Expr {
        let start_idx = self.pos;
        let mut left = self.parse_logical_or();

        while self.consume_if(&TokenKind::ColonColon) {
            let right = self.parse_logical_or();
            left = Expr::Ascription {
                left: Box::new(left),
                right: Box::new(right),
                span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
            };
        }

        left
    }

    fn parse_logical_or(&mut self) -> Expr {
        let start_idx = self.pos;
        let mut left = self.parse_logical_and();
        while self.consume_if(&TokenKind::Or) {
            let right = self.parse_logical_and();
            left = Expr::Binary {
                left: Box::new(left),
                op: ast::BinaryOp::Or,
                right: Box::new(right),
                span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
            };
        }

        left
    }

    fn parse_logical_and(&mut self) -> Expr {
        let start_idx = self.pos;
        let mut left = self.parse_refinement();
        while self.consume_if(&TokenKind::And) {
            let right = self.parse_refinement();
            left = Expr::Binary {
                left: Box::new(left),
                op: ast::BinaryOp::And,
                right: Box::new(right),
                span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
            };
        }

        left
    }

    fn parse_refinement(&mut self) -> Expr {
        let start_idx = self.pos;
        let mut left = self.parse_equality();
        while self.consume_if(&TokenKind::Pipe) {
            // optional binder
            let binder = if let TokenKind::Ident(name) = self.peek_kind() {
                let n = name.clone();
                // lookahead for ->
                // if next token is Arrow then treat as binder
                // otherwise binder is part of expression
                let save = self.pos;
                self.bump();
                if matches!(self.peek_kind(), TokenKind::Arrow) {
                    self.bump();
                    Some(n)
                } else {
                    // rollback
                    self.pos = save;
                    None
                }
            } else {
                None
            };

            let predicate = self.parse_equality();
            left = Expr::Refinement {
                base: Box::new(left),
                binder,
                predicate: Box::new(predicate),
                span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
            };
        }

        left
    }

    fn parse_equality(&mut self) -> Expr {
        let start_idx = self.pos;
        let mut left = self.parse_relational();
        loop {
            if self.consume_if(&TokenKind::EqEq) {
                let right = self.parse_relational();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Eq,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            if self.consume_if(&TokenKind::Ne) {
                let right = self.parse_relational();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Ne,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            break;
        }

        left
    }

    fn parse_relational(&mut self) -> Expr {
        let start_idx = self.pos;
        let mut left = self.parse_additive();
        loop {
            if self.consume_if(&TokenKind::Lt) {
                let right = self.parse_additive();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Lt,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            if self.consume_if(&TokenKind::Le) {
                let right = self.parse_additive();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Le,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            if self.consume_if(&TokenKind::Gt) {
                let right = self.parse_additive();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Gt,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            if self.consume_if(&TokenKind::Ge) {
                let right = self.parse_additive();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Ge,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            break;
        }

        left
    }

    fn parse_additive(&mut self) -> Expr {
        let start_idx = self.pos;
        let mut left = self.parse_multiplicative();
        loop {
            if self.consume_if(&TokenKind::Plus) {
                let right = self.parse_multiplicative();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Add,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            if self.consume_if(&TokenKind::Minus) {
                let right = self.parse_multiplicative();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Sub,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            break;
        }

        left
    }

    fn parse_multiplicative(&mut self) -> Expr {
        let start_idx = self.pos;
        let mut left = self.parse_intersection();
        loop {
            if self.consume_if(&TokenKind::Star) {
                let right = self.parse_intersection();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Mul,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            if self.consume_if(&TokenKind::Slash) {
                let right = self.parse_intersection();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Div,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            if self.consume_if(&TokenKind::Percent) {
                let right = self.parse_intersection();
                left = Expr::Binary {
                    left: Box::new(left),
                    op: ast::BinaryOp::Mod,
                    right: Box::new(right),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                };
                continue;
            }
            break;
        }

        left
    }

    fn parse_intersection(&mut self) -> Expr {
        let start_idx = self.pos;
        let mut left = self.parse_power();
        while self.consume_if(&TokenKind::Amp) {
            let right = self.parse_power();
            left = Expr::Binary {
                left: Box::new(left),
                op: ast::BinaryOp::Intersection,
                right: Box::new(right),
                span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
            };
        }

        left
    }

    fn parse_power(&mut self) -> Expr {
        // right-associative
        let start_idx = self.pos;
        let mut left = self.parse_unary();
        if self.consume_if(&TokenKind::Caret) {
            let right = self.parse_power();
            left = Expr::Binary {
                left: Box::new(left),
                op: ast::BinaryOp::Power,
                right: Box::new(right),
                span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
            };
        }

        left
    }

    fn parse_unary(&mut self) -> Expr {
        let start_idx = self.pos;
        if self.consume_if(&TokenKind::Minus) {
            let expr = self.parse_unary();
            return Expr::Unary {
                op: ast::UnaryOp::Neg,
                expr: Box::new(expr),
                span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
            };
        }
        if self.consume_if(&TokenKind::Not) {
            let expr = self.parse_unary();
            return Expr::Unary {
                op: ast::UnaryOp::Not,
                expr: Box::new(expr),
                span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
            };
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Expr {
        let start_idx = self.pos;
        let mut expr = self.parse_primary();

        loop {
            match self.peek_kind() {
                TokenKind::LParen => {
                    // call
                    self.bump();
                    let mut args = Vec::new();
                    if !matches!(self.peek_kind(), TokenKind::RParen) {
                        loop {
                            args.push(self.parse_verification_expr());
                            if self.consume_if(&TokenKind::Comma) {
                                continue;
                            }
                            break;
                        }
                    }
                    if matches!(self.peek_kind(), TokenKind::RParen) {
                        self.bump();
                    }
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                        span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                    };
                }
                TokenKind::Dot => {
                    // field access; treat right side as identifier
                    self.bump();
                    if let Some(field) = self.expect_ident() {
                        expr = Expr::FieldAccess {
                            base: Box::new(expr),
                            field,
                            span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                        };
                    } else {
                        // unexpected, create error and stop
                        let span = self.current_token_span();
                        self.diagnostics.push(Diagnostic::ExpectedField { span });
                        expr = Expr::Error { span };

                        break;
                    }
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let start_idx = self.pos;
        match self.peek_kind() {
            TokenKind::Int(n) => {
                let v = *n;
                self.bump();

                Expr::LiteralInt {
                    value: v,
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::Real(f) => {
                let v = *f;
                self.bump();

                Expr::LiteralReal {
                    value: v,
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::String(s) => {
                let v = s.clone();
                self.bump();

                Expr::LiteralString {
                    value: v,
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::True => {
                self.bump();

                Expr::LiteralBool {
                    value: true,
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::False => {
                self.bump();

                Expr::LiteralBool {
                    value: false,
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::Hole => {
                self.bump();
                Expr::Hole {
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::DoubleHole => {
                self.bump();

                Expr::Hole {
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.bump();

                Expr::Var {
                    name: n,
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::LParen => {
                self.bump();
                let inner = self.parse_verification_expr();
                if matches!(self.peek_kind(), TokenKind::RParen) {
                    self.bump();
                }

                Expr::Paren {
                    expr: Box::new(inner),
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::LBrace => {
                // block expr
                self.bump();
                let mut statements = Vec::new();
                while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::EOF) {
                    if let Some((stmt, _span)) = self.parse_statement() {
                        statements.push(stmt);
                    } else {
                        // skip token
                        self.bump();
                    }
                }

                // optional tail expr
                let tail = if !matches!(self.peek_kind(), TokenKind::RBrace) {
                    Some(Box::new(self.parse_verification_expr()))
                } else {
                    None
                };

                if matches!(self.peek_kind(), TokenKind::RBrace) {
                    self.bump();
                }

                Expr::Block {
                    statements,
                    tail,
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::If => {
                self.bump();
                let condition = Box::new(self.parse_verification_expr());
                let then_branch = Box::new(self.parse_primary());
                let mut else_branch = None;

                if self.consume_if(&TokenKind::Else) {
                    // else can be block or if (handled by parse_primary)
                    else_branch = Some(Box::new(self.parse_primary()));
                }

                Expr::If {
                    condition,
                    then_branch,
                    else_branch,
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::Match => {
                self.bump();
                let scrutinee = Box::new(self.parse_verification_expr());
                let mut arms = Vec::new();
                if matches!(self.peek_kind(), TokenKind::LBrace) {
                    self.bump();
                }

                while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::EOF) {
                    let pat = self.parse_pattern();
                    if matches!(self.peek_kind(), TokenKind::FatArrow)
                        || matches!(self.peek_kind(), TokenKind::Arrow)
                    {
                        self.bump();
                    }

                    let body = if matches!(self.peek_kind(), TokenKind::LBrace) {
                        // block
                        self.parse_primary()
                    } else {
                        self.parse_verification_expr()
                    };

                    // optional comma
                    if matches!(self.peek_kind(), TokenKind::Comma) {
                        self.bump();
                    }
                    arms.push(MatchArm { pattern: pat, body });
                }

                if matches!(self.peek_kind(), TokenKind::RBrace) {
                    self.bump();
                }

                Expr::Match {
                    scrutinee,
                    arms,
                    span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                }
            }
            TokenKind::Dot => {
                // Variant expression starting with .Identifier
                self.bump();
                if let Some(name) = self.expect_ident() {
                    let mut args = Vec::new();
                    if matches!(self.peek_kind(), TokenKind::LParen) {
                        self.bump();
                        if !matches!(self.peek_kind(), TokenKind::RParen) {
                            loop {
                                args.push(self.parse_verification_expr());
                                if self.consume_if(&TokenKind::Comma) {
                                    continue;
                                }
                                break;
                            }
                        }
                        if matches!(self.peek_kind(), TokenKind::RParen) {
                            self.bump();
                        }
                    }

                    Expr::Variant {
                        name,
                        args,
                        span: self.mark_span_from(start_idx, self.pos.saturating_sub(1)),
                    }
                } else {
                    let span = self.current_token_span();
                    self.diagnostics
                        .push(Diagnostic::ExpectedVariantName { span });

                    Expr::Error { span }
                }
            }
            _ => {
                // unexpected token
                let span = self.current_token_span();
                let kind = self.peek_kind().clone();
                self.diagnostics
                    .push(Diagnostic::UnexpectedToken { kind, token: span });
                self.bump();

                Expr::Error { span }
            }
        }
    }

    fn parse_pattern(&mut self) -> Pattern {
        match self.peek_kind() {
            TokenKind::Int(n) => {
                let v = *n;
                self.bump();

                Pattern::LiteralInt(v)
            }
            TokenKind::Real(f) => {
                let v = *f;
                self.bump();

                Pattern::LiteralReal(v)
            }
            TokenKind::String(s) => {
                let v = s.clone();
                self.bump();

                Pattern::LiteralString(v)
            }
            TokenKind::True => {
                self.bump();

                Pattern::LiteralBool(true)
            }
            TokenKind::False => {
                self.bump();

                Pattern::LiteralBool(false)
            }
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.bump();
                if n == "_" {
                    Pattern::Wildcard
                } else {
                    Pattern::Ident(n)
                }
            }
            TokenKind::Dot => {
                self.bump();
                if let Some(name) = self.expect_ident() {
                    let mut args = Vec::new();
                    if matches!(self.peek_kind(), TokenKind::LParen) {
                        self.bump();
                        if !matches!(self.peek_kind(), TokenKind::RParen) {
                            loop {
                                args.push(self.parse_pattern());
                                if self.consume_if(&TokenKind::Comma) {
                                    continue;
                                }
                                break;
                            }
                        }
                        if matches!(self.peek_kind(), TokenKind::RParen) {
                            self.bump();
                        }
                    }

                    Pattern::Variant { name, args }
                } else {
                    let span = self.current_token_span();
                    self.diagnostics
                        .push(Diagnostic::ExpectedVariantInPattern { span });

                    Pattern::Error
                }
            }

            _ => {
                let span = self.current_token_span();
                self.diagnostics
                    .push(Diagnostic::UnexpectedPattern { pattern: span });
                self.bump();

                Pattern::Error
            }
        }
    }
}

pub async fn parse_program<DB: SyntaxDatabaseTrait>(
    db: &DB,
    source: NamedSource,
) -> PicanteResult<(Program, Vec<Diagnostic>)> {
    let tokens = tokenize(db, source).await?;
    let mut diagnostics = Vec::new();
    for token in &tokens {
        if let TokenKind::Error(error) = &token.kind {
            diagnostics.push(error.diagnostic(token.span));
        }
    }

    let module_id = source.module_id(db)?;
    let mut p = Parser::new(tokens, module_id);
    let stmts = p.parse_program();

    // convert statements+spans into Program
    let statements = stmts.into_iter().map(|(stmt, _span)| stmt).collect();

    diagnostics.extend(p.diagnostics);

    Ok((Program { statements }, diagnostics))
}

pub fn parse_program_source_with_diagnostics(
    src: &str,
    module_id: ModuleId,
) -> (Program, Vec<Diagnostic>) {
    let (tokens, mut diagnostics) = tokenize_with_diagnostics(src, module_id);

    let mut p = Parser::new(tokens, module_id);
    let stmts = p.parse_program();

    // convert statements+spans into Program
    let statements = stmts.into_iter().map(|(stmt, _span)| stmt).collect();
    diagnostics.extend(p.diagnostics);

    (Program { statements }, diagnostics)
}

pub fn parse_program_source(src: &str, module_id: ModuleId) -> Program {
    let (program, _) = parse_program_source_with_diagnostics(src, module_id);

    program
}
