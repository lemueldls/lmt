# lmt-solver - SMT Solver Integration

Backend abstraction for SMT constraint solving; converts type-checker constraints into SMT-LIB2 format and
invokes solvers.

## Architecture

The crate provides an IR for SMT expressions and abstracts over solver backends (string-based or native bindings). The
checker emits verification conditions that the solver evaluates.

## Key Features

- SMT IR and backend abstraction
- String-based backend and feature-gated native bindings
- Quantifier and sort support with extension points

## Usage

APIs allow constructing SMT expressions, invoking a solver backend, and interpreting results. See crate documentation
for examples.

## Notes

- Native bindings are preferable for interactive use due to performance
- The solver interface is designed to be pluggable so other backends can be added
