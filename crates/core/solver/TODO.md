# TODO - lmt-solver (SMT Solver Integration)

## Known Limitations

- **String-based solver is default** - Using external process is slow; native CVC5 bindings are optional feature
- **Limited sort support** - Only Int and Bool sorts; missing Arrays, BitVectors, Strings
- **No quantifier optimization** - Quantifiers are passed directly to solver; no instantiation hints
- **No synthesis support** - Cannot solve SyGuS (Syntax-Guided Synthesis) problems for `??` holes
- **No model extraction** - Cannot retrieve counterexample/model from solver when unsat
- **Error handling is basic** - Solver crashes not gracefully handled; no timeout support
- **No incremental solver session** - Each check_validity creates new solver context (expensive)

## Planned Features

### Short Term

- [ ] Native CVC5 as default
  - [ ] Enable `cvc5` feature by default
  - [ ] Make string-based solver optional fallback
  - [ ] Profile and compare performance
- [ ] Timeout support
  - [ ] Add time budget parameter to solver calls
  - [ ] Return Unknown if solver times out
  - [ ] Make timeout configurable (CLI flag, environment variable)
- [ ] Better error reporting
  - [ ] Catch solver crashes and report gracefully
  - [ ] Provide solver stderr in diagnostics
  - [ ] Suggest workarounds (e.g., "solver unavailable, cannot verify")

### Medium Term

- [ ] Additional sorts
  - [ ] BitVector sort (for hardware verification, byte literals)
  - [ ] Array sort (for data structure verification)
  - [ ] String sort (for string constraint solving)
- [ ] Incremental solver session
  - [ ] Maintain persistent solver context across multiple checks
  - [ ] Assert environment once, reuse for multiple subtype checks
  - [ ] Significant performance improvement for type-checking large programs
- [ ] Model extraction
  - [ ] When check fails, retrieve counterexample from solver
  - [ ] Use counterexample in error messages (e.g., "5 is not positive")
  - [ ] Enable interactive debugging via LSP

### Long Term

- [ ] Synthesis support
  - [ ] Convert `??` holes to SyGuS problems
  - [ ] Invoke cvc5 synthesis engine
  - [ ] Generate code for synthesized expressions
- [ ] Quantifier optimization
  - [ ] Provide instantiation hints to solver
  - [ ] Use E-matching or other techniques to speed up quantifier reasoning
- [ ] Custom solver strategies
  - [ ] Allow user-defined solver tactics via configuration file
  - [ ] Example: "use aggressive quantifier instantiation for this proof"

## Performance / Optimization

- [ ] Solver batching
  - [ ] Group multiple subtype checks into single solver call
  - [ ] Reduce solver invocation overhead
- [ ] Caching of solver results
  - [ ] Memoize is_subtype queries (they're deterministic)
  - [ ] Hash-based lookup in cache
- [ ] String-based solver optimization
  - [ ] Generate more compact SMT-LIB2 strings
  - [ ] Parallelize external solver invocation if multiple checks pending
- [ ] Memory profiling
  - [ ] Profile solver IR memory usage
  - [ ] Consider expression sharing/interning

## Testing Gaps

- [ ] Solver correctness tests
  - [ ] Verify solver correctly decides satisfiability
  - [ ] Test against known satisfiable/unsatisfiable formulas
- [ ] Correctness of IR generation
  - [ ] Compare SMT-LIB2 output with hand-written formulas
  - [ ] Verify no unsoundness in expression lowering
- [ ] Solver availability tests
  - [ ] Test graceful degradation if solver not installed
  - [ ] Test timeout behavior
- [ ] Performance benchmarks
  - [ ] Benchmark native CVC5 vs. string-based
  - [ ] Benchmark incremental vs. fresh context

## Integration TODOs

- **Checker interface** - Finalize API between lmt-checker and solver (may need batch operations)
- **CVC5 feature flag** - Coordinate build system to prefer native CVC5 when available
- **Solver selection** - Allow users to choose between CVC5 and Z3 (future)
- **Error propagation** - Ensure solver errors bubble up through checker and diagnostics layers

## Notes

- Solver performance is critical; performance issues here directly impact editor responsiveness
- Incremental solver context is major win but requires careful state management
- Synthesis (?? holes) is a cool feature but lower priority than basic verification
- Consider publishing solver IR as intermediate format for other uses
