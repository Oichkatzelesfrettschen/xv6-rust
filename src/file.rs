use crate::fs::NDIRECT;
use crate::pipe::Pipe;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
/// Open file description.
pub struct File {
    /// File type (see file.c).
    itype: i32,
    /// Reference count.
    refc: i32,
    /// Readable flag.
    readable: u8,
    /// Writable flag.
    writable: u8,
    /// Pipe for pipe files.
    pipe: *const Pipe,
    /// Inode for device or file.
    ip: *const Inode,
    /// File offset.
    off: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
/// On-disk inode structure.
pub struct Inode {
    /// Device number.
    dev: u32,
    /// Inode number.
    inum: u32,
    /// Reference count in memory.
    refc: u32,
    /// Is inode data valid?
    valid: i32,
    /// File type.
    itype: i16,
    /// Major device number (T_DEV only).
    major: i16,
    /// Number of links to inode in file system.
    nlink: i16,
    /// Size of file (bytes).
    size: u32,
    /// Data block addresses.
    addrs: [u32; NDIRECT + 1],
}
