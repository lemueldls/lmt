# lmt-lsp - Language Server Protocol

LSP server enabling IDE and editor integration for LMT.

## Architecture

The server implements the Language Server Protocol and runs as a separate process communicating via JSON-RPC. It manages
incremental updates, invokes the type checker, and publishes diagnostics.

```
Editor (VS Code/Zed)
   ↓ (JSON-RPC over stdio)
LSP Server
   ├─ Document Manager
   ├─ Text Synchronization
   ├─ Type Checker Integration
   └─ Diagnostics Publisher
   ↓ (Notifications)
Editor (Real-time Feedback)
```

## Key Features

- Text synchronization for opened and changed documents
- Hover information with inferred types
- Go-to-definition for basic symbol navigation
- Real-time diagnostics via `PublishDiagnostics`

## Usage

The server is launched by an editor client. For debugging, run:

```bash
cargo run --bin lmt-lsp
```

## Notes

- The server integrates with the incremental parser and type checker to reduce work on edits
- Diagnostic publishing follows the LSP `PublishDiagnostics` convention
