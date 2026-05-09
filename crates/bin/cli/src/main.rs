use anyhow::{Result, anyhow};
use facet::Facet;
use figue::{self as args, FigueBuiltins};
use lmt_checker::check;

#[derive(Facet, Debug)]
struct Cli {
    #[facet(args::subcommand)]
    command: Option<Command>,

    #[facet(flatten)]
    builtins: FigueBuiltins,
}

#[derive(Facet, Debug)]
#[repr(u8)]
enum Command {
    /// Check proofs in a project
    Check {
        /// Path to the project or file to check
        #[facet(args::positional)]
        path: String,
    },
}

fn main() -> Result<()> {
    let outcome = figue::from_std_args::<Cli>();
    let output = outcome.into_result().map_err(|e| anyhow!("{:?}", e))?;
    let args = output.value;

    match args.command {
        Some(Command::Check { path }) => {
            println!("Checking project at: {}", path);
            check()?;
        }
        None => {
            println!("LMT: Refinement Type Proof Assistant");
            println!("Use --help for usage information.");
        }
    }

    Ok(())
}
