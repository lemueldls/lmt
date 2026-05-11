//! Simple file system abstraction for checking files from various sources.
//!
//! Supports reading from real disk or in-memory buffers, useful for both
//! CLI (file paths) and LSP (open editor buffers).

use std::{collections::HashMap, path::Path, sync::Arc};

use parking_lot::RwLock;

/// File system interface for reading and writing files.
pub trait FileSystem: Send + Sync {
    /// Read entire file contents
    fn read_file(&self, path: &str) -> Result<Vec<u8>, String>;

    /// Write file contents
    fn write_file(&self, path: &str, contents: &[u8]) -> Result<(), String>;
}

/// Real file system backed by OS disk.
#[derive(Clone, Copy)]
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path, e))
    }

    fn write_file(&self, path: &str, contents: &[u8]) -> Result<(), String> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }
        std::fs::write(path, contents)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }
}

/// In-memory file system for LSP buffers and testing.
#[derive(Clone)]
pub struct MemoryFileSystem {
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for MemoryFileSystem {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        self.files
            .read()
            .get(path)
            .cloned()
            .ok_or_else(|| format!("File not found: {}", path))
    }

    fn write_file(&self, path: &str, contents: &[u8]) -> Result<(), String> {
        self.files
            .write()
            .insert(path.to_string(), contents.to_vec());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_filesystem() {
        let fs = MemoryFileSystem::new();
        let path = "test.lmt";
        let content = b"let x = 42";

        // Write
        fs.write_file(path, content).unwrap();

        // Read
        let read = fs.read_file(path).unwrap();
        assert_eq!(read, content);
    }

    #[test]
    fn test_real_filesystem() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("lmt_fs_test_123.txt");
        let path_str = test_file.to_str().unwrap();

        let fs = RealFileSystem;
        let content = b"test content";

        // Write
        fs.write_file(path_str, content).unwrap();

        // Read
        let read = fs.read_file(path_str).unwrap();
        assert_eq!(read, content);

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }
}
