use std::{
    // collections::HashMap,
    fs,
    // hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

// use ahash::{AHashMap, AHasher};
// use ariadne::{sources, Color, Label, Report, ReportKind};
// use directories::ProjectDirs;
// use hashbrown::Equivalent;
use lmt_report::miette::{self, IntoDiagnostic, NamedSource};
use lmt_synthesis::{Graph, ModuleId, ModuleSynthesis, StaticSynthesis};

// #[derive(Debug, Default)]
// pub struct FsGraph {
//     module_synthesis_map: HashMap<PathBuf, ModuleSynthesis>,
// }

// impl Graph for FsGraph {
//     fn resolve_path(&mut self, path: &str) -> Option<String> {
//         // let path = PathBuf::from(path).canonicalize().ok()?;

//         fs::read_to_string(path).ok()
//     }
// }

fn main() -> miette::Result<()> {
    miette::set_panic_hook();

    let synthesis = StaticSynthesis::default();
    // let filename = env::args().nth(1).expect("no file given");

    let path = "examples/test.lmt";
    let src = fs::read_to_string(path).into_diagnostic()?;

    let (module_id, errors) = synthesis.load_module(path, &src);

    for error in errors {
        eprintln!(
            "{:?}",
            error.report_with_source(path.to_string(), src.to_string())
        )
    }

    Ok(())
}
