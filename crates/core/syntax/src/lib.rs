//! lmt-syntax: incremental parser/lexer and semantic analysis framework (scaffold)

pub mod ast;
pub mod db;
pub mod diagnostic;
pub mod input;
pub mod lexer;
pub mod lsp;
pub mod parser;
pub mod semantic;
pub mod token;

pub use db::Database;
pub use input::SourceFile;
pub use token::{Span, Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_basic_tokens() {
        use lexer::Lexer;
        use token::TokenKind;

        let src = "1 abc;";
        let mut lx = Lexer::new(src);
        let t1 = lx.next_token();
        assert!(matches!(t1.kind, TokenKind::Int(1)));
        let t2 = lx.next_token();
        match t2.kind {
            TokenKind::Ident(ref s) => assert_eq!(s, "abc"),
            _ => panic!("expected ident"),
        }
        let t3 = lx.next_token();
        assert!(matches!(t3.kind, TokenKind::Semi));
        let t4 = lx.next_token();
        assert!(matches!(t4.kind, TokenKind::EOF));
    }

    #[test]
    fn parser_recovery_simple() {
        use parser::parse_program;

        let src = "1 @@@ abc ; 2";
        let program = parse_program(src);
        // expect at least one Error expr and later a LiteralInt(2)
        let mut has_error = false;
        let mut has_two = false;
        for stmt in program.statements.iter() {
            if let ast::Statement::Expr(expr) = stmt {
                match expr {
                    ast::Expr::Error(_) => has_error = true,
                    ast::Expr::LiteralInt(2) => has_two = true,
                    _ => {}
                }
            }
        }
        assert!(has_error, "recovery should produce an Error expr");
        assert!(has_two, "parser should recover and parse trailing 2");
    }
}
