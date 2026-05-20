# TODO - lmt-index (Indexing Infrastructure)

## Known Limitations

- **Index types are opaque** - Cannot inspect macro-generated index types; hard to debug
- **No validation** - Index types don't validate that indices are within bounds (out-of-bounds returns None)
- **No garbage collection** - Deleted entries stay in slot map; manual cleanup required
- **RwLock overhead** - SlotLockMap acquires lock on every access; single-threaded workloads pay overhead
- **Limited secondary maps** - SecondarySlotLockMap only supports basic 2D lookups
- **No arena allocator** - No batch allocation for performance; each insert is individual
- **Index generation not customizable** - `define_index_type!` macro always generates same interface

## Planned Features

### Short Term

- [ ] Better macro debugging
  - [ ] Generate docs for macro-created types
  - [ ] Make macro expansion inspectable with `cargo expand`
  - [ ] Add examples in macro documentation
- [ ] Validation layer (optional)
  - [ ] Compile-time or runtime bounds checking
  - [ ] Debug mode that tracks allocations
- [ ] Specialized allocators
  - [ ] Arena allocator for batch insertions
  - [ ] Generation-based allocator for safe reuse

### Medium Term

- [ ] Enhanced secondary maps
  - [ ] Support 3+ dimensional lookups
  - [ ] Iterators over secondary key ranges
  - [ ] Better API ergonomics
- [ ] Concurrent collections
  - [ ] Lock-free variants for high-contention scenarios
  - [ ] Compare-and-swap based updates
- [ ] Serialization improvements
  - [ ] Binary serialization (not just u64 FFI)
  - [ ] Schema versioning for compatibility

### Long Term

- [ ] Specialized index types
  - [ ] Dense bitset indices for compact storage
  - [ ] Sparse matrix indices for data structures
- [ ] Macro customization
  - [ ] Allow users to derive additional traits on index types
  - [ ] Custom serialization formats
- [ ] Performance-focused variants
  - [ ] Cache-friendly layout options
  - [ ] SIMD-friendly batch operations

## Performance / Optimization

- [ ] Benchmark against alternatives
  - [ ] Compare SlotLockMap vs. HashMap vs. Vec
  - [ ] Profile memory layout and cache utilization
- [ ] RwLock alternatives
  - [ ] Experiment with parking_lot vs. std RwLock
  - [ ] Try lock-free designs for read-heavy workloads
  - [ ] Measure contention in real workloads
- [ ] Memory layout optimization
  - [ ] Analyze slot map fragmentation
  - [ ] Measure average entry size and alignment
- [ ] Batch operations
  - [ ] Optimize multi-insert performance
  - [ ] Test insert rate limits

## Testing Gaps

- [ ] Macro expansion tests
  - [ ] Verify generated code is correct
  - [ ] Test edge cases (single-letter names, reserved words in paths)
- [ ] Index type tests
  - [ ] Type safety tests (cannot mix index types)
  - [ ] FFI serialization roundtrip
- [ ] SlotLockMap tests
  - [ ] Insert/get/remove correctness
  - [ ] Out-of-bounds handling (None returns)
  - [ ] Concurrent access tests
- [ ] Stress tests
  - [ ] Large numbers of indices (millions)
  - [ ] Memory usage profiling
  - [ ] Long-lived servers (memory leaks check)

## Integration TODOs

- **Cross-crate usage** - Currently ModuleId defined in lmt-diagnostics; consider moving to lmt-index
- **AST node indexing** - Reserve ExprId, StatementId types for future AST indexing use
- **Performance monitoring** - Add telemetry for index operations in production

## Notes

- This is a foundational utility crate; changes here affect all downstream crates
- Macro system is powerful but hard to debug; invest in documentation and examples
- RwLock provides thread safety but may not be needed everywhere; consider opt-in feature
- Consider publishing index crate separately as open-source utility
