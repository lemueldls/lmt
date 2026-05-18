use std::{fs, path::PathBuf};

use lmt_diagnostics::{
    ModuleId,
    graph::{ModuleGraph, VirtualGraph},
};
use lmt_syntax::{Database, db::tokenize_source, parser::parse_program_source};
use walkdir::WalkDir;

fn render_tokens(src: &str, module_id: ModuleId) -> String {
    let tokens = tokenize_source(src, module_id);
    let mut out = String::new();

    for token in tokens {
        out.push_str(&format!(
            "{:?}@{}..{}\n",
            token.kind,
            token.span.start().unwrap(),
            token.span.end().unwrap()
        ));
    }

    out
}

fn render_case(src: &str) -> String {
    let db = Database::new();
    let mut graph = VirtualGraph::new();

    let module_id = graph.register(&db, "test.lmt");
    graph.set_content(&db, module_id, src.to_string()).unwrap();

    let program = parse_program_source(src, module_id);
    format!(
        "SOURCE:\n{src}\n\nTOKENS:\n{}\nPROGRAM:\n{program:#?}",
        render_tokens(src, module_id)
    )
}

#[test]
fn examples() {
    let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut cases = Vec::new();

    for entry in WalkDir::new(&examples_dir).min_depth(1).max_depth(1) {
        let entry = entry.expect("failed to read example directory entry");
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("lmt") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("example file should have a UTF-8 stem")
            .to_string();
        let src = fs::read_to_string(path).expect("failed to read example source");
        cases.push((name, src));
    }

    cases.sort_by(|left, right| left.0.cmp(&right.0));

    for (name, src) in cases {
        insta::assert_snapshot!(name, render_case(&src));
    }
}
