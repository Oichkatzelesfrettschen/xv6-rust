use crate::fs::NDIRECT;
use crate::pipe::Pipe;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct File {
    itype: i32, // should be enum
    refc: i32,
    readable: u8,
    writable: u8,
    pipe: *const Pipe,
    ip: *const Inode,
    off: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Inode {
    dev: u32,
    inum: u32,
    refc: u32,
    // SLEEP LOCK
    valid: i32,
    itype: i16,
    major: i16,
    nlink: i16,
    size: u32,
    addrs: [u32; NDIRECT + 1],
}
