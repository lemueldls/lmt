use anyhow::{Result, anyhow};
use facet::Facet;
use figue::{self as args, FigueBuiltins};
use lmt_checker::check_path;

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
}

fn main() -> Result<()> {
    let cli: Cli = figue::from_std_args().unwrap();

    match cli.command {
        Command::Check { path } => {
            println!("Checking project at: {}", path);
            check_path(&path)?;
        }
    }

    Ok(())
}
