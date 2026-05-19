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
    /// If the file does not exist, returns an empty [`ConfigFile`] (same as
    /// [`ConfigFile::new`]). Use [`ConfigFile::minimal`] if you need a
    /// pre-populated config.
    pub fn read_default() -> Result<Self, IoError> {
        let path = Path::new(SELINUX_CONFIG_PATH);
        match Self::read_from(path) {
            Ok(cfg) => Ok(cfg),
            Err(e) => {
                if e.source.kind() == std::io::ErrorKind::NotFound {
                    Ok(ConfigFile::new())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Atomically write the config to the given path.
    ///
    /// The write is performed atomically via a unique temporary file:
    /// 1. Serialize the config to a uniquely-named `.tmp` sibling of the target
    /// 2. Call `fsync` on the temporary file
    /// 3. Rename the temporary file over the target
    /// 4. Call `fsync` on the parent directory to ensure the rename is durable
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), IoError> {
        let path = path.as_ref();
        let content = self.to_string();

        // Use a unique temp file name to prevent races between concurrent writers
        let tmp_name = format!(
            ".{}.tmp",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("config")
        );
        let tmp_path = path.with_file_name(tmp_name);

        fs::write(&tmp_path, &content).map_err(|e| IoError {
            path: tmp_path.clone(),
            source: e,
        })?;

        let tmp_file = fs::File::open(&tmp_path).map_err(|e| IoError {
            path: tmp_path.clone(),
            source: e,
        })?;
        tmp_file.sync_all().map_err(|e| IoError {
            path: tmp_path.clone(),
            source: e,
        })?;

        fs::rename(&tmp_path, path).map_err(|e| IoError {
            path: path.to_path_buf(),
            source: e,
        })?;

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
