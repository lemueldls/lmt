//! Incremental parser/lexer

pub mod ast;
pub mod db;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod token;

pub use db::SyntaxDatabase;
pub use lmt_diagnostics::source::NamedSource;
pub use token::{Token, TokenKind};

#[cfg(test)]
mod tests {
    use lmt_diagnostics::graph::{ModuleGraph, VirtualGraph};

    use super::*;

    #[test]
    fn parser_recovery_simple() {
        use parser::parse_program_source;

        let db = SyntaxDatabase::new();
        let mut graph = VirtualGraph::new();

        let content = "1 @@@ abc ; 2";

        let module_id = graph.upsert(&db, "test.lmt", content.to_string());

        let program = parse_program_source(content, module_id);
        // expect at least one Error expr and later a LiteralInt(2)
        let mut has_error = false;
        let mut has_two = false;
        for stmt in program.statements.iter() {
            if let ast::Statement::Expr(expr) = stmt {
                match expr {
                    ast::Expr::Error { .. } => has_error = true,
                    ast::Expr::LiteralInt { value: 2, .. } => has_two = true,
                    _ => {}
                }
            }
        }
        assert!(has_error, "recovery should produce an Error expr");
        assert!(has_two, "parser should recover and parse trailing 2");
    }
}
