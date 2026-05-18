use anyhow::Result;
use facet::Facet;
use figue::{self as args, FigueBuiltins};
use lmt_diagnostics::graph::{FsGraph, ModuleGraph};
use lmt_syntax::{Database, parser::parse_program};

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
    /// Check proofs in a project
    Check {
        /// Path to the project or file to check
        #[facet(args::positional)]
        path: String,
    },

    /// Evaluates an expression and prints the result
    Eval {
        /// The expression to evaluate
        #[facet(args::positional)]
        expr: String,
    },

    /// Run a project
    Run {
        /// Path to the project or file to run
        #[facet(args::positional)]
        path: String,
    },

    /// Start a REPL session
    Repl,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = figue::from_std_args().unwrap();

    let db = Database::new();
    let mut graph = FsGraph::new();

    match cli.command {
        Command::Check { path } => {
            println!("Checking project at: {}", path);

            let module_id = graph.register(&db, &path);
            let source = graph.get(module_id);
            let (program, diagnostics) = parse_program(&db, *source).await?;

            dbg!(program);

            for diag in diagnostics {
                diag.report(&db, &graph);
            }
        }
        Command::Eval { expr } => {
            println!("Evaluating expression: {}", expr);
        }
        Command::Run { path } => {
            println!("Running project at: {}", path);
        }
        Command::Repl => {
            println!("Starting REPL session...");
        }
    }

    Ok(())
}
