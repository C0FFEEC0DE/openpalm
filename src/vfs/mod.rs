//! Virtual File System (VFS) for Palm OS
//!
//! This module provides VFS operations for accessing files on Palm devices.
//! VFS allows reading and writing files on expansion cards.

use crate::types::VfsFileAttributes;
use crate::types::{FourCharCode, PalmDateTime, VfsOpenMode};

/// Volume reference number
pub type VolumeRef = u16;

/// File reference number
pub type FileRef = u32;

/// Re-export for convenience
pub use crate::types::VfsVolumeAttributes;

/// VFS volume information
#[derive(Debug, Clone)]
pub struct VolumeInfo {
    /// Volume reference
    pub volume_ref: VolumeRef,
    /// Mount flags
    pub mount_flags: u16,
    /// Volume attributes
    pub attributes: VfsVolumeAttributes,
    /// Media type (FourCC)
    pub media_type: FourCharCode,
    /// Volume name
    pub name: String,
    /// Primary VFS header block
    pub vfs_header: u32,
    /// Number of files
    pub file_count: u32,
    /// Size of volume
    pub total_size: u32,
    /// Free space
    pub free_space: u32,
    /// Reserved
    pub reserved: u32,
}

/// VFS directory entry
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// Entry attributes
    pub attributes: VfsFileAttributes,
    /// File name
    pub name: String,
    /// Local volume ID
    pub volume_id: u32,
    /// Local ID
    pub local_id: u32,
    /// Creation time
    pub created: PalmDateTime,
    /// Modification time
    pub modified: PalmDateTime,
    /// Backup time
    pub backup_date: PalmDateTime,
    /// File size
    pub size: u32,
    /// File type (for resources)
    pub file_type: FourCharCode,
}

/// VFS file
pub struct VfsFile {
    /// File reference
    pub file_ref: FileRef,
    /// File name
    pub name: String,
    /// Open mode
    pub mode: VfsOpenMode,
    /// Current position
    pub position: u32,
    /// File size
    pub size: u32,
    /// Open for reading
    pub readable: bool,
    /// Open for writing
    pub writable: bool,
}

/// VFS implementation helper
// VFS path utilities
pub mod path {
    /// Maximum path length
    pub const MAX_PATH: usize = 256;

    /// Path separator
    pub const SEPARATOR: char = '/';

    /// Check if path is absolute
    pub fn is_absolute(path: &str) -> bool {
        path.starts_with(SEPARATOR)
    }

    /// Get parent directory
    pub fn parent(path: &str) -> Option<&str> {
        let trimmed = path.trim_end_matches(SEPARATOR);
        trimmed.rfind(SEPARATOR).map(|i| &trimmed[..i])
    }

    /// Get file/directory name
    pub fn file_name(path: &str) -> Option<&str> {
        let trimmed = path.trim_end_matches(SEPARATOR);
        trimmed
            .rfind(SEPARATOR)
            .map(|i| trimmed.get(i + 1..).unwrap_or(trimmed))
    }

    /// Join path components
    pub fn join(a: &str, b: &str) -> String {
        let a = a.trim_end_matches(SEPARATOR);
        format!("{}{}{}", a, SEPARATOR, b)
    }

    /// Normalize path (remove duplicate separators)
    pub fn normalize(path: &str) -> String {
        let mut result = String::new();
        let mut prev_sep = false;

        for ch in path.chars() {
            if ch == SEPARATOR {
                if !prev_sep {
                    result.push(ch);
                    prev_sep = true;
                }
            } else {
                result.push(ch);
                prev_sep = false;
            }
        }

        result.trim_end_matches(SEPARATOR).to_string()
    }

    /// Split path into components
    pub fn components(path: &str) -> Vec<&str> {
        path.split(SEPARATOR).filter(|s| !s.is_empty()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VfsFileAttributes;

    #[test]
    fn test_vfs_file_attributes() {
        let attrs = VfsFileAttributes::READ_ONLY | VfsFileAttributes::HIDDEN;
        assert!(attrs.contains(VfsFileAttributes::READ_ONLY));
        assert!(attrs.contains(VfsFileAttributes::HIDDEN));
        assert!(!attrs.intersects(VfsFileAttributes::DIRECTORY));
    }

    #[test]
    fn test_path_utils() {
        assert!(path::is_absolute("/Palm/Programs/test.txt"));
        assert!(!path::is_absolute("test.txt"));

        assert_eq!(path::file_name("/Palm/Programs/test.txt"), Some("test.txt"));
        assert_eq!(
            path::parent("/Palm/Programs/test.txt"),
            Some("/Palm/Programs")
        );

        assert_eq!(path::join("/Palm", "Programs"), "/Palm/Programs");
        assert_eq!(
            path::normalize("/Palm///Programs///test.txt"),
            "/Palm/Programs/test.txt"
        );
    }

    #[test]
    fn test_path_components() {
        let components = path::components("/Palm/Programs/TestDB.pdb");
        assert_eq!(components.len(), 3);
        assert_eq!(components[0], "Palm");
        assert_eq!(components[2], "TestDB.pdb");
    }
}
