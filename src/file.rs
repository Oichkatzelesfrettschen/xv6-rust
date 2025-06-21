//! \file file.rs
//! \brief Kernel file and inode structures.

use crate::fs::NDIRECT;
use crate::pipe::Pipe;
use bytemuck::Zeroable;
use zerocopy::{FromBytes, IntoBytes as AsBytes};

/// \brief Open file description (in-memory).
#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Zeroable)]
pub struct File {
    /// \brief File type (see file.c).
    pub itype:    i32,
    /// \brief Reference count.
    pub refc:     i32,
    /// \brief Read permission flag.
    pub readable: u8,
    /// \brief Write permission flag.
    pub writable: u8,
    /// \brief Back pointer to pipe structure if this is a pipe.
    pub pipe:     *const Pipe,
    /// \brief Inode backing the file.
    pub ip:       *const Inode,
    /// \brief Current offset within file (bytes).
    pub off:      u32,
}

/// \brief On-disk inode structure.
#[repr(C)]
#[repr(packed)]
#[derive(Debug, Copy, Clone, Default, FromBytes, AsBytes)]
pub struct Inode {
    /// \brief Device number containing the inode.
    pub dev:    u32,
    /// \brief Inode number.
    pub inum:   u32,
    /// \brief In-memory reference count.
    pub refc:   u32,
    /// \brief Indicates whether this inode's data is valid.
    pub valid:  i32,
    /// \brief Inode type (file, directory, etc.).
    pub itype:  i16,
    /// \brief Major device number (for device files).
    pub major:  i16,
    /// \brief Number of links to this inode in the filesystem.
    pub nlink:  i16,
    /// \brief Size of file in bytes.
    pub size:   u32,
    /// \brief Data block addresses (direct plus one indirect).
    pub addrs:  [u32; NDIRECT + 1],
}
