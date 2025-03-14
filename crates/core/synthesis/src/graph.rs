use lmt_parser::Spanned;

use crate::ModuleId;

pub trait Graph: Default {
    fn load_path(&mut self, path: &str, module_id: ModuleId);

    fn resolve_path(&self, path: &str) -> ModuleId;

    // fn parse(src: &str) {
    //     let (tokens, errs) = lmt_parser::lexer().parse(src).into_output_errors();

    //     let (module_synthesis, tokenize_errors) = if let Some(tokens) = tokens.as_ref() {
    //         let len = src.chars().count();
    //         let (ast, parse_errs) = lmt_parser::module_parser()
    //             .parse(tokens.spanned((len..len).into()))
    //             .into_output_errors();

    //         let module_synthesis = ast.map(|stmts| SYNTHESIS.synthesize(0, stmts));

    //         (module_synthesis, parse_errs)
    //     } else {
    //         (None, Vec::new())
    //     };

    //     // dbg!(synthesis);

    //     let parse_errors = errs
    //         .into_iter()
    //         .map(|err| err.map_token(|ch| ch.to_string()))
    //         .chain(
    //             tokenize_errors
    //                 .into_iter()
    //                 .map(|err| err.map_token(|token| token.to_string())),
    //         )
    //         .map(|err| {
    //             Report::build(ReportKind::Error, filename, err.span().start)
    //                 .with_message(err.to_string())
    //                 .with_label(
    //                     Label::new((filename, err.span().into_range()))
    //                         .with_message(err.reason().to_string())
    //                         .with_color(Color::Red),
    //                 )
    //                 .with_labels(err.contexts().map(|(label, span)| {
    //                     Label::new((filename, span.into_range()))
    //                         .with_message(format!("while parsing this {label}"))
    //                         .with_color(Color::Blue)
    //                 }))
    //                 .finish()
    //         });
    // }

    // fn synthesize(stmts: Vec<Spanned<Stmt>>) -> Synthesis {
    //     let mut synthesis = Synthesis::default();

    //     for stmt in stmts {
    //         synthesis.eval_statement(stmt);
    //     }

    //     synthesis
    // }
}
