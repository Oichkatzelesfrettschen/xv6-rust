//! \file file.rs
//! \brief Kernel file and inode structures.

use crate::fs::NDIRECT;
use crate::pipe::Pipe;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// \brief Open file handle.
pub struct File {
    /// File type (would be an enum in pure Rust)
    itype: i32,
    /// Reference count
    refc: i32,
    /// Read permission flag
    readable: u8,
    /// Write permission flag
    writable: u8,
    /// Back pointer to pipe structure if this is a pipe
    pipe: *const Pipe,
    /// Inode backing the file
    ip: *const Inode,
    /// Current offset within file
    off: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// \brief On-disk inode structure.
pub struct Inode {
    /// Device containing the inode
    dev: u32,
    /// Inode number
    inum: u32,
    /// Reference count
    refc: u32,
    /// Indicates whether this inode has been read from disk
    valid: i32,
    /// Inode type
    itype: i16,
    /// Major device number (for device files)
    major: i16,
    /// Number of links to inode in filesystem
    nlink: i16,
    /// Size of file in bytes
    size: u32,
    /// Data block addresses
    addrs: [u32; NDIRECT + 1],
}
