//! zed-lmt - Zed editor extension for LMT

use zed_extension_api::{self as zed, Result};

struct LmtExtension;

impl zed::Extension for LmtExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let path = if let Ok(path) = std::env::var("LMT_LSP_PATH") {
            path
        } else {
            worktree.which("lmt-lsp").ok_or_else(|| {
                "lmt-lsp not found in PATH. Please install it and ensure it is available in your PATH or set LMT_LSP_PATH.".to_string()
            })?
        };

        Ok(zed::Command {
            command: path,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(LmtExtension);
