//! File I/O for [`ConfigFile`]: atomic reads and writes with
//! [`read_from`](ConfigFile::read_from), [`write_to`](ConfigFile::write_to),
//! and the [`SELINUX_CONFIG_PATH`] constant.

use crate::config_file::ConfigFile;
use crate::error::IoError;
use std::fs;
use std::io::Read;
use std::path::Path;

/// The default SELinux configuration file path.
pub const SELINUX_CONFIG_PATH: &str = "/etc/selinux/config";

impl ConfigFile {
    /// Read and parse an SELinux config file from the given path.
    ///
    /// Opens the file, reads its content to a string, and parses it using
    /// [`ConfigFile::parse`].  Returns an `IoError` if the file cannot be
    /// read or if the content is invalid.
    pub fn read_from(path: impl AsRef<Path>) -> Result<Self, IoError> {
        let path = path.as_ref();
        let mut file = fs::File::open(path).map_err(|e| IoError {
            path: path.to_path_buf(),
            source: e,
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| IoError {
            path: path.to_path_buf(),
            source: e,
        })?;
        ConfigFile::parse(&content).map_err(|e| IoError {
            path: path.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
        })
    }

    /// Read the default SELinux config file (`/etc/selinux/config`).
    ///
    /// If the file does not exist, returns an empty [`ConfigFile`] (equivalent
    /// to [`ConfigFile::new()`]).
    pub fn read_default() -> Result<Self, IoError> {
        let path = Path::new(SELINUX_CONFIG_PATH);
        if !path.exists() {
            return Ok(ConfigFile::new());
        }
        Self::read_from(path)
    }

    /// Atomically write the config to the given path.
    ///
    /// The write is performed atomically:
    /// 1. Serialize the config to a `.tmp` sibling of the target path
    /// 2. Call `fsync` on the temporary file
    /// 3. Rename the temporary file over the target
    /// 4. Call `fsync` on the parent directory to ensure the new name is durable
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), IoError> {
        let path = path.as_ref();
        let tmp_path = path.with_extension("tmp");
        let content = self.to_string();

        fs::write(&tmp_path, &content).map_err(|e| IoError {
            path: tmp_path.clone(),
            source: e,
        })?;

        // fsync the temporary file
        let tmp_file = fs::File::open(&tmp_path).map_err(|e| IoError {
            path: tmp_path.clone(),
            source: e,
        })?;
        tmp_file.sync_all().map_err(|e| IoError {
            path: tmp_path.clone(),
            source: e,
        })?;

        // Atomic rename
        fs::rename(&tmp_path, path).map_err(|e| IoError {
            path: path.to_path_buf(),
            source: e,
        })?;

        // fsync the parent directory
        if let Some(parent) = path.parent() {
            let dir = fs::File::open(parent).map_err(|e| IoError {
                path: parent.to_path_buf(),
                source: e,
            })?;
            dir.sync_all().map_err(|e| IoError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        Ok(())
    }

    /// Atomically write the config to the default path (`/etc/selinux/config`).
    pub fn write_default(&self) -> Result<(), IoError> {
        self.write_to(SELINUX_CONFIG_PATH)
    }
}
