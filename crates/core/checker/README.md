# lmt-checker - Type Checker and Semantic Analysis

Bidirectional type-checking engine that performs semantic analysis and generates verification conditions for
SMT solvers.

## Architecture

The checker implements a bidirectional type system with synthesis and checking modes. Refinement types pair base types
with logical predicates verified by the SMT solver.

```
AST (from lmt-syntax)
    ↓
Type Environment
    ↓
Verification Conditions
    ↓
Solver
    ↓
Diagnostics / Results
```

## Key Features

- Bidirectional type checking for improved inference and error messages
- Refinement types verified by an SMT solver
- Subtype checking delegated to the solver
- Incremental checking via Picante integration

## Usage

Core APIs are provided for parsing programs, running checks, and querying diagnostics from the checker database. See
crate documentation for examples.

## Notes

- Refinement types are central to LMT and are the main focus of this crate
- The checker formulates proof obligations for the solver and converts solver responses into diagnostics
