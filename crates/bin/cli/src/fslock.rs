use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
    process,
};

// Used for locking.
use fs4::FileExt;
use serde::{Deserialize, Serialize};
use simplelock::{ConcreteLock, LockStatus, SimpleLockError, SimpleLockResult};
// Used to push lock details into the file.
use uuid::Uuid;

/// A simple cross-platform file lock.
pub struct FileLock {
    /// Per object/thread/process ids.
    id: FileLockId,

    /// Path we are using for the file. Considered consistent across threads/processes.
    path: PathBuf,

    /// File handle for locking/unlocking on.
    file: File,

    /// Current process/thread knows about lock status and can speak with authority.
    have_lock: Option<bool>,
}

/// The FileLockId is a unique ID generated for each "FileLock" object.
/// It combines the ProcessID and a UUID so that it can be validated
/// by a casual viewing of the locked file and a current process list.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct FileLockId {
    /// Per process id.
    pid: u32,

    /// Per object/thread id.
    uid: Uuid,
}

impl Default for FileLockId {
    fn default() -> Self {
        FileLockId {
            pid: process::id(),
            uid: Uuid::new_v4(),
        }
    }
}

impl FileLock {
    /// Create a new Lock given the path/name of a file we can lock on.
    pub fn new<P: Into<PathBuf>>(pathbuf: P) -> SimpleLockResult<FileLock> {
        let path = pathbuf.into();
        let id = FileLockId::default();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(SimpleLockError::from)?;

        Ok(FileLock {
            id,
            path,
            file,
            have_lock: None,
        })
    }

    /// Get the actual path this file lock is using.
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl ConcreteLock for FileLock {
    fn status(&self) -> SimpleLockResult<LockStatus> {
        match self.have_lock {
            Some(true) => Ok(LockStatus::Mine),
            _ => {
                if self.file.allocated_size().map_or(false, |v| v > 0) {
                    Ok(LockStatus::Taken)
                } else {
                    Ok(LockStatus::Open)
                }
            }
        }
    }

    fn try_lock(&mut self) -> SimpleLockResult<()> {
        if self.have_lock.unwrap_or(false) {
            return Ok(());
        }

        self.file
            .try_lock_exclusive()
            .and_then(|_| serde_json::to_writer(&self.file, &self.id).map_err(|e| e.into()))
            .map_err(|e| e.into())
            .map(|_| {
                self.have_lock = Some(true);
            })
    }

    fn hang_lock(&mut self) -> SimpleLockResult<()> {
        if self.have_lock.unwrap_or(false) {
            return Ok(());
        }

        self.file
            .lock_exclusive()
            .and_then(|_| serde_json::to_writer(&self.file, &self.id).map_err(|e| e.into()))
            .map_err(|e| e.into())
            .map(|_| {
                self.have_lock = Some(true);
            })
    }

    fn try_unlock(&mut self) -> SimpleLockResult<()> {
        if self.have_lock.unwrap_or(true) {
            return Ok(());
        }

        // Set length to 0 effectively deletes the content of the file.
        self.file.set_len(0)?;
        self.file.unlock().map_err(|e| e.into())
    }
}
