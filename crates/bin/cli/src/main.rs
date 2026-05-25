use anyhow::Result;
use facet::Facet;
use figue::{self as args, FigueBuiltins};
use lmt_checker::{CheckerDatabase, check_program};
use lmt_diagnostics::graph::{FsGraph, ModuleGraph as _};
use lmt_syntax::{SyntaxDatabase, parser::parse_program};

#[derive(Facet)]
struct Cli {
    #[facet(args::subcommand)]
    command: Command,

    #[facet(flatten)]
    builtins: FigueBuiltins,
}

#[derive(Facet)]
#[repr(u8)]
enum Command {
    /// Check proofs in a project.
    Check {
        /// Path to the project or file to check.
        #[facet(args::positional)]
        path: String,
    },

    /// Evaluates an expression and prints the result.
    Eval {
        /// The expression to evaluate.
        #[facet(args::positional)]
        expr: String,
    },

    /// Run a project.
    Run {
        /// Path to the project or file to run.
        #[facet(args::positional)]
        path: String,
    },

    /// Start a REPL session.
    Repl,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = figue::from_std_args().unwrap();

    let syntax_db = SyntaxDatabase::new();
    let checker_db = CheckerDatabase::new();
    let mut graph = FsGraph::new();

    match cli.command {
        Command::Check { path } => {
            let module_id = graph.upsert_path(&syntax_db, &path);
            let source = graph.get(module_id);
            let (program, mut diagnostics) = parse_program(&syntax_db, *source).await?;
            drop(source);

            // Run semantic checking
            if let Err(checker_diags) = check_program(&checker_db, program).await? {
                diagnostics.extend(checker_diags);
            }

            for diag in diagnostics {
                diag.print(&syntax_db, &graph);
            }
        }
        Command::Eval { expr } => {
            println!("Evaluating expression: {expr}");
        }
        Command::Run { path } => {
            println!("Running project at: {path}");
        }
        Command::Repl => {
            println!("Starting REPL session...");
        }
    }

    Ok(())
}
