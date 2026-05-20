# TODO - lmt-syntax (Parser and Abstract Syntax Tree)

## Known Limitations

- **Parser error recovery is basic** - Stops at some complex syntax errors instead of continuing
- **No detailed error messages** - Error diagnostics lack context or recovery suggestions
- **Token windowing incomplete** - Performance benefit not yet fully exploited for very large files
- **Limited precedence customization** - Operator precedence is hardcoded; no way to define custom operators
- **No source location precision** - Spans include full expressions, not individual tokens
- **Pattern matching limited** - Only basic patterns; no or-patterns or nested destructuring
- **Type annotations are expressions** - Type system not formalized in parser (no distinct Type vs. Expr)

## Planned Features

### Short Term

- [ ] Enhanced error recovery
  - [ ] Skip to next statement/expression on parse error
  - [ ] Provide suggestions for common mistakes (missing :, = instead of ==)
  - [ ] Recover from unterminated strings/blocks
- [ ] Better diagnostics
  - [ ] Point to exact token causing error
  - [ ] Suggest fixes (e.g., "expected :" when : is missing)
  - [ ] Include context lines in error output
- [ ] Token hygiene
  - [ ] Track exact token positions (not just byte offsets)
  - [ ] Enable precise error highlighting in editors

### Medium Term

- [ ] Advanced patterns
  - [ ] Or-patterns (p1 | p2)
  - [ ] Nested destructuring in match arms
  - [ ] Wildcard patterns with binding guards
- [ ] Custom operators (design phase)
  - [ ] Allow users to define operators with custom precedence
  - [ ] Parser generator improvements for extensibility
- [ ] Type annotations as first-class syntax
  - [ ] Separate Type grammar from Expr grammar
  - [ ] Enable richer type syntax (type aliases, type parameters)

### Long Term

- [ ] Macro expansion support (design phase)
- [ ] Template literals / string interpolation
- [ ] Comments as first-class AST nodes (for documentation)
- [ ] Byte offsets to source ranges (richer spans)

## Performance / Optimization

- [ ] Incremental parsing scalability - Test performance on multi-megabyte files
- [ ] Token windowing efficiency
  - [ ] Profile windowing algorithm
  - [ ] Measure cache hit rates
  - [ ] Optimize window boundary selection
- [ ] Parser memory usage
  - [ ] Profile AST size for typical programs
  - [ ] Consider tree compression or serialization
- [ ] Lexer performance
  - [ ] Profile tokenization speed on large files
  - [ ] Optimize string scanning and keyword recognition

## Testing Gaps

- [ ] Parser roundtrip tests (parse → format → parse → equal)
- [ ] Error recovery tests
  - [ ] Test each error scenario with recovery
  - [ ] Verify parser continues to next valid construct
- [ ] Edge cases
  - [ ] Maximum nesting depth
  - [ ] Very long identifiers
  - [ ] Unicode identifiers (if supported)
- [ ] Regression tests
  - [ ] Keep test cases for bugs found and fixed
  - [ ] Ensure error messages are stable

## Integration TODOs

- **Diagnostic annotation** - Coordinate with lmt-diagnostics for better error reporting
- **Incremental parsing verification** - Validate that incremental results match full parse
- **Type checker handoff** - Ensure AST format is optimal for lmt-checker consumption
- **Language server updates** - Work with lmt-lsp to optimize incremental updates

## Notes

- Parser design prioritizes error recovery over performance (pragmatic trade-off)
- Token windowing is powerful but complex; document the algorithm clearly
- Recursive descent parser is maintainable; consider code generation only if grammar grows significantly
