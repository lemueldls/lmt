# TODO - lmt-cli (Command-Line Interface)

## Known Limitations

- **`eval` command is a stub** - Currently no-op; needs expression parsing, type checking, and result formatting
- **`run` command is a stub** - Needs multi-file project support and code execution (or interpretation)
- **`repl` command is a stub** - No implementation; needs line editing (rustyline?), state management, and eval loop
- **Project manifest support missing** - No Cargo.toml or lmt.toml parsing; assumes single-file projects
- **No module resolution** - Cannot import other .lmt files via `use` statements
- **Limited error context** - Error output doesn't include helpful recovery hints

## Planned Features

### Short Term

- [ ] Implement `eval` command to evaluate single expressions
  - [ ] Parse and type-check expression
  - [ ] Format and display result
  - [ ] Handle refinement types in output
- [ ] Implement `run` command for program execution
  - [ ] Load and check entire project
  - [ ] Execute entry point (TBD: main() function convention)
  - [ ] Capture and format output

### Medium Term

- [ ] Implement basic REPL with line editing (rustyline crate)
  - [ ] Read-Eval-Print loop
  - [ ] Persistent context across commands
  - [ ] Multi-line expression support
- [ ] Project manifest support (lmt.toml or Cargo.toml)
  - [ ] Parse manifest
  - [ ] Resolve dependencies
  - [ ] Configure entry point

### Long Term

- [ ] Incremental compilation mode (watch for changes)
- [ ] Benchmark/profile subcommand
- [ ] Interactive proof explorer for refinement types
- [ ] Plugin/extension system for custom commands

## Performance / Optimization

- [ ] Async I/O for large projects (already using tokio, but not exploited)
- [ ] Parallel type-checking for multiple files (needs rayon integration)
- [ ] Caching of parsed/checked modules across runs
- [ ] Faster source file discovery (currently scans directories eagerly)

## Testing Gaps

- [ ] Integration tests for `check` command with various input files
- [ ] Error message consistency tests (ensure helpful diagnostics)
- [ ] Large project stress tests (performance baseline)
- [ ] REPL interaction tests (line editing, multi-line expressions)
- [ ] CLI argument parsing edge cases

## Integration TODOs

- **Module resolution** - Depends on lmt-diagnostics module graph completion
- **Project manifest** - Need to define lmt.toml schema or integrate with Cargo.toml
- **Interpreter/Executor** - Need to decide: interpret AST or compile to intermediate representation?
- **Output formatting** - Align with LSP diagnostics format for consistency

## Notes

- REPL should share type-checking state with main CLI for consistency
- `eval` and `run` are conceptually different; eval is ephemeral, run is project-scoped
