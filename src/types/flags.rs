//! Bitflags for Palm OS protocols
//!
//! This module contains all the flag types used in Palm OS protocols.

use bitflags::bitflags;

bitflags! {
    /// Record attributes from Palm device
    ///
    /// These flags indicate the state of a record:
    /// - Whether it's been modified
    /// - Whether it's marked for deletion
    /// - Whether it's secret (hidden)
    /// - etc.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct RecordFlags: u8 {
        /// Record is marked for deletion during next sync
        const DELETED = 0x80;
        /// Record has been modified since last sync
        const DIRTY = 0x40;
        /// Record is busy (in use)
        const BUSY = 0x20;
        /// Record is secret (hidden from normal view)
        const SECRET = 0x10;
        /// Record is archived
        const ARCHIVED = 0x08;
    }
}

impl RecordFlags {
    /// Check if the record is marked for deletion
    pub fn is_deleted(&self) -> bool {
        self.contains(RecordFlags::DELETED)
    }

    /// Check if the record has been modified
    pub fn is_dirty(&self) -> bool {
        self.contains(RecordFlags::DIRTY)
    }

    /// Check if the record is busy
    pub fn is_busy(&self) -> bool {
        self.contains(RecordFlags::BUSY)
    }

    /// Check if the record is secret
    pub fn is_secret(&self) -> bool {
        self.contains(RecordFlags::SECRET)
    }

    /// Check if the record is archived
    pub fn is_archived(&self) -> bool {
        self.contains(RecordFlags::ARCHIVED)
    }

    /// Check if the record is modified (any change)
    pub fn is_modified(&self) -> bool {
        self.intersects(RecordFlags::DIRTY | RecordFlags::DELETED)
    }
}

bitflags! {
    /// Database flags
    ///
    /// These flags describe the properties of a database.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct DatabaseFlags: u16 {
        /// Resource database (not a record database)
        const RESOURCE = 0x0001;
        /// Database is read-only
        const READ_ONLY = 0x0002;
        /// AppInfo block has been modified
        const APP_INFO_DIRTY = 0x0004;
        /// Database should be backed up during HotSync
        const BACKUP = 0x0008;
        /// Newer version may be installed over open DB (Palm OS 2.0+)
        const NEWER = 0x0010;
        /// Reset after installation (Palm OS 2.0+)
        const RESET = 0x0020;
        /// Copy prevention - cannot be beamed (Palm OS 3.0+)
        const COPY_PREVENTION = 0x0040;
        /// Database is a file stream (Palm OS 3.0+)
        const STREAM = 0x0080;
        /// Database is hidden
        const HIDDEN = 0x0100;
        /// Database is launchable (show in Launcher, launch by Creator)
        const LAUNCHABLE = 0x0200;
        /// Database will be deleted shortly
        const RECYCLABLE = 0x0400;
        /// Bundled with others having same creator (for Beam)
        const BUNDLE = 0x0800;
        /// Database is currently open
        const OPEN = 0x8000;
        /// Schema database (Palm OS 6.0+)
        const SCHEMA = 0x1000;
        /// Secure database (Palm OS 6.0+)
        const SECURE = 0x2000;
        /// Fixed up - temp flag used to clear DB on write (Palm OS 6.0+)
        const FIXED_UP = 0x4000;
        /// Exclude from sync (DLP 1.1+)
        const EXCLUDE_FROM_SYNC = 0x80;
        /// RAM-based database (DLP 1.2+)
        const RAM_BASED = 0x40;
    }
}

impl DatabaseFlags {
    /// Check if database is a resource database
    pub fn is_resource(&self) -> bool {
        self.contains(DatabaseFlags::RESOURCE)
    }

    /// Check if database is read-only
    pub fn is_read_only(&self) -> bool {
        self.contains(DatabaseFlags::READ_ONLY)
    }

    /// Check if database should be backed up
    pub fn should_backup(&self) -> bool {
        self.contains(DatabaseFlags::BACKUP)
    }

    /// Check if database is hidden
    pub fn is_hidden(&self) -> bool {
        self.contains(DatabaseFlags::HIDDEN)
    }

    /// Check if database is launchable
    pub fn is_launchable(&self) -> bool {
        self.contains(DatabaseFlags::LAUNCHABLE)
    }

    /// Check if database is RAM-based
    pub fn is_ram_based(&self) -> bool {
        self.contains(DatabaseFlags::RAM_BASED)
    }
}

bitflags! {
    /// Open database mode flags
    ///
    /// Used with dlp_OpenDB to specify how to open the database.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct OpenMode: u8 {
        /// Open database for reading
        const READ = 0x80;
        /// Open database for writing
        const WRITE = 0x40;
        /// Open database with exclusive access
        const EXCLUSIVE = 0x20;
        /// Show secret records
        const SECRET = 0x10;
    }
}

impl OpenMode {
    /// Read-only mode
    pub fn read() -> Self {
        OpenMode::READ
    }

    /// Write-only mode
    pub fn write() -> Self {
        OpenMode::WRITE
    }

    /// Read and write mode
    pub fn read_write() -> Self {
        OpenMode::READ.union(OpenMode::WRITE)
    }

    /// Check if read mode is enabled
    pub fn can_read(&self) -> bool {
        self.contains(OpenMode::READ)
    }

    /// Check if write mode is enabled
    pub fn can_write(&self) -> bool {
        self.contains(OpenMode::WRITE)
    }

    /// Check if exclusive mode is enabled
    pub fn is_exclusive(&self) -> bool {
        self.contains(OpenMode::EXCLUSIVE)
    }

    /// Check if secret mode is enabled
    pub fn show_secrets(&self) -> bool {
        self.contains(OpenMode::SECRET)
    }
}

bitflags! {
    /// Database list filter flags
    ///
    /// Used with dlp_ReadDBList to filter which databases to return.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct DatabaseListFlags: u8 {
        /// List RAM databases
        const RAM = 0x80;
        /// List ROM databases
        const ROM = 0x40;
        /// List as many databases as possible (DLP 1.2+)
        const MULTIPLE = 0x20;
    }
}

bitflags! {
    /// VFS file open mode flags
    ///
    /// Used with dlp_VFSFileOpen.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct VfsOpenMode: u8 {
        /// Exclusive access
        const EXCLUSIVE = 0x01;
        /// Read only
        const READ = 0x02;
        /// Write only (implies exclusive)
        const WRITE = 0x04;
        /// Read and write
        const READ_WRITE = 0x07;
        /// Create file if it doesn't exist
        const CREATE = 0x08;
        /// Truncate to 0 bytes on open
        const TRUNCATE = 0x10;
        /// Leave file open even if foreground task closes
        const LEAVE_OPEN = 0x20;
    }
}

impl VfsOpenMode {
    /// Read-only mode
    pub fn read_only() -> Self {
        VfsOpenMode::READ
    }

    /// Write-only mode (with exclusive and create)
    pub fn write_only() -> Self {
        VfsOpenMode::WRITE
            .union(VfsOpenMode::EXCLUSIVE)
            .union(VfsOpenMode::CREATE)
    }

    /// Read-write mode
    pub fn read_write() -> Self {
        VfsOpenMode::READ_WRITE
    }
}

bitflags! {
    /// VFS file attributes
    ///
    /// File/directory attributes returned by dlp_VFSDirEntryEnumerate.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct VfsFileAttributes: u32 {
        /// File is read only
        const READ_ONLY = 0x00000001;
        /// File is hidden
        const HIDDEN = 0x00000002;
        /// File is a system file
        const SYSTEM = 0x00000004;
        /// File is the volume label
        const VOLUME_LABEL = 0x00000008;
        /// File is a directory
        const DIRECTORY = 0x00000010;
        /// File is archived
        const ARCHIVE = 0x00000020;
        /// File is a link to another file
        const LINK = 0x00000040;
    }
}

impl VfsFileAttributes {
    /// Check if this is a directory
    pub fn is_directory(&self) -> bool {
        self.contains(VfsFileAttributes::DIRECTORY)
    }

    /// Check if this is a regular file
    pub fn is_file(&self) -> bool {
        !self.intersects(VfsFileAttributes::DIRECTORY | VfsFileAttributes::VOLUME_LABEL)
    }
}

bitflags! {
    /// VFS volume attributes
    ///
    /// Volume attributes returned by dlp_VFSVolumeInfo.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct VfsVolumeAttributes: u32 {
        /// Volume is inserted in an expansion slot
        const SLOT_BASED = 0x00000001;
        /// Volume is read-only
        const READ_ONLY = 0x00000002;
        /// Volume is hidden
        const HIDDEN = 0x00000004;
    }
}

impl VfsVolumeAttributes {
    /// Check if volume is read-only
    pub fn is_read_only(&self) -> bool {
        self.contains(VfsVolumeAttributes::READ_ONLY)
    }

    /// Check if volume is removable (slot-based)
    pub fn is_removable(&self) -> bool {
        self.contains(VfsVolumeAttributes::SLOT_BASED)
    }
}

bitflags! {
    /// VFS seek origin constants
    ///
    /// Used with dlp_VFSFileSeek.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct VfsSeekOrigin: u8 {
        /// From the beginning (first data byte of file)
        const BEGINNING = 0;
        /// From the current position
        const CURRENT = 1;
        /// From the end of file (one position beyond last data byte)
        const END = 2;
    }
}

bitflags! {
    /// VFS file date types
    ///
    /// Used with dlp_VFSFileGetDate and dlp_VFSFileSetDate.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct VfsDateType: u8 {
        /// The date the file was created
        const CREATED = 1;
        /// The date the file was last modified
        const MODIFIED = 2;
        /// The date the file was last accessed
        const ACCESSED = 3;
    }
}

/// Category ID type
pub type categoryid_t = u8;

/// Category ID constants
pub mod category {
    use super::categoryid_t;

    /// Unfiled category (all uncategorized records)
    pub const UNFILED: categoryid_t = 0;

    /// Delete category
    pub const DELETE: categoryid_t = 0xFE;

    /// Palm OS category IDs are 0-15 for user categories
    pub const USER_CATEGORY_MIN: categoryid_t = 0;
    pub const USER_CATEGORY_MAX: categoryid_t = 15;

    /// Check if a category ID is valid
    pub fn is_valid(id: categoryid_t) -> bool {
        id <= 15 || id == DELETE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_flags() {
        let flags = RecordFlags::DELETED | RecordFlags::DIRTY;
        assert!(flags.is_deleted());
        assert!(flags.is_dirty());
        assert!(!flags.is_secret());

        assert_eq!(flags.bits(), 0xC0);
    }

    #[test]
    fn test_record_flags_roundtrip() {
        let original = RecordFlags::from_bits_truncate(0xD8);
        assert!(original.contains(RecordFlags::DELETED));
        assert!(original.contains(RecordFlags::DIRTY));
        assert!(original.contains(RecordFlags::SECRET));
        assert!(original.contains(RecordFlags::ARCHIVED));
    }

    #[test]
    fn test_database_flags() {
        let flags = DatabaseFlags::READ_ONLY | DatabaseFlags::BACKUP;
        assert!(flags.is_read_only());
        assert!(flags.should_backup());
        assert!(!flags.is_hidden());
    }

    #[test]
    fn test_open_mode() {
        let mode = OpenMode::read_write();
        assert!(mode.can_read());
        assert!(mode.can_write());

        let read_only = OpenMode::READ;
        assert!(read_only.can_read());
        assert!(!read_only.can_write());
    }

    #[test]
    fn test_vfs_open_mode() {
        let mode = VfsOpenMode::read_write();
        assert!(mode.contains(VfsOpenMode::READ));
        assert!(mode.contains(VfsOpenMode::WRITE));
    }

    #[test]
    fn test_vfs_file_attrs() {
        let attrs = VfsFileAttributes::DIRECTORY;
        assert!(attrs.is_directory());
        assert!(!attrs.is_file());

        let file_attrs = VfsFileAttributes::READ_ONLY | VfsFileAttributes::ARCHIVE;
        assert!(file_attrs.is_file());
    }

    #[test]
    fn test_category_valid() {
        assert!(category::is_valid(0));
        assert!(category::is_valid(15));
        assert!(category::is_valid(0xFE));
        assert!(!category::is_valid(16));
        assert!(!category::is_valid(0xFF));
    }
}
