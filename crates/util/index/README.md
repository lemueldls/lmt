# lmt-index - Indexing Infrastructure

Low-level indexing primitives for type-safe, thread-safe indexed access.

## Architecture

Provides macros to generate newtype index wrappers and slot map based storage primitives for efficient lookups.

## Key Features

- Type-safe index newtypes via `define_index_type!`
- SlotLockMap for thread-safe dense storage
- Secondary maps for multi-dimensional lookups

## Usage

Define index types and use slot maps for efficient storage. See crate documentation for examples.

## Notes

- This crate is foundational and used by other LMT components
- SlotLockMap uses locks for safety; alternatives may be preferred for single-threaded workloads

## Planned Extensions

- Performance benchmarking against alternative data structures
- Secondary key optimization for multi-dimensional lookups
- Specialized arena allocator for batch insertions
