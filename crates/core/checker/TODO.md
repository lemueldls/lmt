# TODO - lmt-checker (Type Checker and Semantic Analysis)

## Known Limitations

- **Refinement type support is incomplete** - Basic support exists but lacks:
  - Refinement weakening (contravariance in refinement predicates)
  - Intersection and union types
  - Refinement narrowing in match arms
- **No dependent function types** - Cannot express functions that return types depending on arguments
- **Sum types (variants) are stubs** - Pattern matching on variants not fully implemented
- **Product types (records) are missing** - No struct/record type support
- **Limited scope tracking** - Environment doesn't handle nested scopes perfectly (for loop bodies, match arms)
- **No polymorphism** - No generic types or type parameters
- **Solver integration is basic** - Doesn't use solver's advanced features (quantifiers, function uninterpreting, etc.)

## Planned Features

### Short Term

- [ ] Refine subtype checking
  - [ ] Implement refinement weakening correctly
  - [ ] Handle intersection types (T1 & T2)
  - [ ] Support union types (T1 | T2) separately from refinement syntax
- [ ] Improve error messages
  - [ ] Explain why refinement failed (show counterexample from solver)
  - [ ] Suggest type narrowing or assertions
- [ ] Better environment management
  - [ ] Track all bindings through nested scopes
  - [ ] Validate no shadowing (or warn about it)

### Medium Term

- [ ] Sum types / variants
  - [ ] Type-check variant construction
  - [ ] Exhaustiveness checking for match arms
  - [ ] Type narrowing after pattern matching
- [ ] Product types / records
  - [ ] Struct type definitions
  - [ ] Field access type checking
  - [ ] Record update type checking
- [ ] Polymorphism (generic types)
  - [ ] Type parameters on functions and types
  - [ ] Constraint solving for type inference with generics
  - [ ] Type instantiation at call sites

### Long Term

- [ ] Dependent function types
  - [ ] Return type depends on argument values
  - [ ] Refinement types as dependent types
- [ ] Implicit arguments
  - [ ] Infer implicit parameters from context
- [ ] Higher-ranked types (rank-N polymorphism)
- [ ] Type classes or similar constraint resolution

## Performance / Optimization

- [ ] Incremental type-checking
  - [ ] Cache type inference results (per Picante tracked function)
  - [ ] Only re-check expressions that changed or depend on changes
- [ ] Solver performance
  - [ ] Batch SMT queries instead of one-at-a-time
  - [ ] Cache solver responses (is_subtype is deterministic)
  - [ ] Add time budget/timeout for solver calls
- [ ] Memory usage
  - [ ] Profile environment size for deeply nested programs
  - [ ] Consider arena allocation for type nodes

## Testing Gaps

- [ ] Type inference tests
  - [ ] Comprehensive examples of inferred types
  - [ ] Edge cases (deeply nested expressions, complex precedence)
- [ ] Refinement type tests
  - [ ] Valid refinements that checker should accept
  - [ ] Invalid refinements that checker should reject
  - [ ] Solver correctness tests
- [ ] Error message tests
  - [ ] Ensure error messages are helpful
  - [ ] Check that diagnostics point to correct locations
- [ ] Regression tests for bugs found and fixed
- [ ] Large program stress tests

## Integration TODOs

- **Solver interface** - Finalize API between checker and solver for batch queries
- **AST format** - Ensure parser produces optimal AST for checker (may need tweaks)
- **Diagnostics propagation** - All checker errors should flow through lmt-diagnostics
- **Incremental tracking** - Coordinate with Picante to track which queries change on edits
- **LSP integration** - Ensure checker provides enough info for LSP features (hover, definition)

## Notes

- Type checking is the core of LMT; this crate is the highest priority for correctness
- Refinement types are the main innovation; focus on getting them right before adding polymorphism
- Solver integration is critical path for verification; performance here is high priority
- Consider publishing type system formally (type judgments, subtyping rules) for clarity
