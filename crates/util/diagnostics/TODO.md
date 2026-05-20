# TODO - lmt-diagnostics (Error Reporting and Module Management)

## Known Limitations

- **Module graph is abstraction-only** - Trait defined but implementations (FsGraph, LspGraph, VirtualGraph) need
  robustness work
- **No streaming diagnostics** - All diagnostics collected before rendering; large error lists could be streamed
- **Line-column lookup is eager** - Byte offsets converted to line:column on demand; no caching
- **Rendering is single-threaded** - No parallelization when generating output for many diagnostics
- **Limited error recovery hints** - Diagnostics don't suggest fixes or workarounds
- **No colored diffs** - Output doesn't show before/after for transformations
- **Module IDs are opaque** - No way to inspect or debug module graph structure
- **LSP conversion is manual** - Each Report must be manually converted to LSP Diagnostic

## Planned Features

### Short Term

- [ ] Streaming diagnostics
  - [ ] Stream diagnostics to output as they're generated (not buffered)
  - [ ] Enable progress feedback for long-running checks
- [ ] Better error recovery hints
  - [ ] Suggest fixes for common mistakes (e.g., "Did you mean...?")
  - [ ] Point to similar patterns in codebase
  - [ ] Include code examples
- [ ] Caching for line-column lookups
  - [ ] Cache byte→(line, col) mappings per source file
  - [ ] LRU cache with size limit
  - [ ] Measure hit rate

### Medium Term

- [ ] Enhanced rendering
  - [ ] Colored diffs for before/after transformations
  - [ ] Breadcrumb navigation for nested scopes
  - [ ] Inline code examples in error messages
- [ ] Module graph robustness
  - [ ] Handle file deletions gracefully (LspGraph)
  - [ ] Track unsaved changes vs. on-disk
  - [ ] Handle symlinks and circular dependencies
- [ ] Automatic Report → LSP Diagnostic conversion
  - [ ] Implement From<Report> trait
  - [ ] Make conversion lossless where possible

### Long Term

- [ ] Interactive error explorer (LSP-based)
  - [ ] Browse error tree
  - [ ] Show related errors together
  - [ ] Drill into error causes
- [ ] Contextual hints
  - [ ] Learn common mistakes from user history
  - [ ] Personalize suggestions
- [ ] Multilingual diagnostics (i18n)
  - [ ] Render messages in user's language
  - [ ] Keep error context in English for debugging

## Performance / Optimization

- [ ] Diagnostic rendering parallelization
  - [ ] Use rayon to render multiple diagnostics in parallel
  - [ ] Profile and measure speedup
- [ ] Memory usage
  - [ ] Profile diagnostics storage for large projects
  - [ ] Consider lazy string construction
- [ ] Line-column lookup acceleration
  - [ ] Build byte offset index upfront
  - [ ] Binary search instead of linear scan
- [ ] Module graph query performance
  - [ ] Cache module lookups
  - [ ] Index modules by name for faster search

## Testing Gaps

- [ ] Module graph tests
  - [ ] Test each ModuleGraph implementation (FsGraph, LspGraph, VirtualGraph)
  - [ ] Test module resolution and caching
  - [ ] Test error handling (missing files, permission denied, etc.)
- [ ] Rendering tests
  - [ ] Snapshot tests for error output formatting
  - [ ] Test fancy vs. plain rendering modes
  - [ ] Verify ANSI codes are correct
- [ ] Line-column accuracy tests
  - [ ] Test with Unicode, multi-byte characters
  - [ ] Test with different line endings (CRLF, LF)
  - [ ] Test with very long lines
- [ ] LSP conversion tests
  - [ ] Ensure Report→LSP Diagnostic roundtrip preserves information
  - [ ] Test LSP client rendering of converted diagnostics

## Integration TODOs

- **Module graph abstraction** - Ensure all compiler phases use trait instead of concrete implementations
- **Span precision** - Work with parser to provide fine-grained spans (per-token, not per-expression)
- **LSP diagnostics** - Ensure all diagnostics from lmt-checker/lmt-syntax convert correctly to LSP format
- **CLI rendering** - Coordinate with lmt-cli for consistent error output

## Notes

- Diagnostics are user-facing; quality here directly impacts developer experience
- Module graph trait is elegant abstraction; ensure it covers all use cases (CLI, LSP, testing)
- Line-column conversion is frequent operation; performance here matters for large files
- Consider publishing diagnostic rendering as separate library for LMT documentation/tooling
