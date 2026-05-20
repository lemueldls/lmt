# TODO - lmt-lsp (Language Server Protocol)

## Known Limitations

- **No code completion** - `textDocument/completion` not implemented
- **No inlay hints** - Type information not shown inline in editor
- **No refactoring** - Rename, extract function, etc. not available
- **No semantic tokens** - Syntax highlighting is basic (relies on textmate grammar)
- **Limited symbol navigation** - Go-to-definition works but no document symbols or workspace symbol search
- **Slow incremental updates** - Full re-parse on each change; doesn't exploit windowed tokenization
- **No caching** - Diagnostics recalculated even for unchanged expressions
- **Notification batching missing** - May send diagnostics too frequently under rapid edits

## Planned Features

### Short Term

- [ ] Optimize incremental checking
  - [ ] Use Picante change tracking to avoid re-checking unchanged code
  - [ ] Batch diagnostic notifications (debounce)
  - [ ] Caching of type inference results
- [ ] Semantic token support
  - [ ] Generate LSP SemanticToken for identifiers, keywords, type names
  - [ ] Enable syntax highlighting in editors (VS Code, Zed)
- [ ] Code completion (basic)
  - [ ] Complete variable names in scope
  - [ ] Suggest type constructors and refinements

### Medium Term

- [ ] Workspace symbol support (`workspace/symbol` request)
  - [ ] Index all symbols in open files
  - [ ] Enable cross-file navigation
- [ ] Signature help
  - [ ] Display function signatures on hover during function calls
- [ ] Hover improvements
  - [ ] Show full type with refinement constraints
  - [ ] Include source span and documentation

### Long Term

- [ ] Code actions (refactoring)
  - [ ] Quick fixes for type errors
  - [ ] Suggest refinement constraints
- [ ] Rename (symbol refactoring)
- [ ] Extract function/let binding
- [ ] Inlay hints
  - [ ] Show inferred types inline
  - [ ] Show refinement constraints

## Performance / Optimization

- [ ] Concurrency improvement - Handle multiple document updates in parallel without blocking
- [ ] Memory usage - Profile and optimize server memory for large projects
- [ ] LSP message batching - Combine diagnostics, hover info in single response
- [ ] Slow operations timeout - Set time budget for type-checking before reporting timeout
- [ ] Caching strategy - Implement smart caching (LRU, fingerprint-based)

## Testing Gaps

- [ ] End-to-end LSP protocol tests
  - [ ] Simulate editor client for realistic sequences
  - [ ] Test error recovery under malformed input
- [ ] Concurrent edit tests (rapid changes from multiple clients)
- [ ] Large file stress tests
- [ ] Diagnostic rendering tests (ensure LSP messages are valid)
- [ ] Memory leak tests (long-running server stability)

## Integration TODOs

- **Incremental parsing integration** - Exploit lmt-syntax Picante DB for faster re-parsing
- **Solver optimization** - Use native cvc5 bindings (feature flag) instead of string-based solver
- **Module graph robustness** - Handle edge cases in LspGraph (file deletion, unsaved changes)
- **Error context** - Propagate rich diagnostic info from lmt-checker through LSP layer

## Notes

- LSP server is stateful; needs careful state management for concurrent clients
- Notifications (diagnostics) are async; responses (hover, definition) must be quick
- Consider implementing debouncing for rapid edits (e.g., 500ms after last keystroke)
- Each editor (VS Code, Zed, Neovim) has different capabilities; may need capability negotiation
