# lmt-diagnostics - Error Reporting and Module Management

Error and diagnostic reporting infrastructure, source span tracking, and module graph abstraction.

## Architecture

Provides a `ModuleGraph` abstraction and `Report` structures for rendering diagnostics in different formats.

## Key Features

- Span management for precise source locations
- ModuleGraph trait with filesystem and in-memory backends
- Multiple rendering formats including plain text and terminal output

## Usage

Create `Report` values and render them with the provided printers. The crate exposes helpers for converting reports to
LSP diagnostics.

## Notes

- ModuleGraph enables different backends (filesystem, LSP, testing)
- Diagnostic rendering supports plain and colored output via optional features
