# lmt-cli - Command-Line Interface

Standalone command-line tool for checking, evaluating, and running LMT programs.

## Architecture

The CLI is the primary entry point for users outside editors. It routes subcommands to the parser, type checker, and
solver, then formats diagnostics for output.

```
User Input (CLI Arguments)
    ↓
Command Router (check, eval, run, repl)
    ↓
lmt-checker (Type Checking)
    ↓
lmt-solver (SMT Verification)
    ↓
Formatted Output / Diagnostics
```

## Key Features

- `check <path>` - Parse and type-check LMT code and report diagnostics
- `eval <expr>` - Evaluate a single LMT expression (stub)
- `run <path>` - Compile and execute an LMT program (stub)
- `repl` - Interactive REPL for exploring expressions (stub)

## Usage

```bash
# Check a file
lmt check path/to/program.lmt

# Evaluate an expression
lmt eval "1 + 2"

# Run a program
lmt run path/to/program.lmt

# Start REPL
lmt repl
```

## Notes

- The CLI is designed to follow the same diagnostics format as the LSP server
- Multi-file support and REPL are planned but not yet fully implemented
