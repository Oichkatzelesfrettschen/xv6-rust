/// Number of bytes in a pipe buffer.
const PIPESIZE: usize = 512;

#[repr(C)]
/// Pipe buffer with reader/writer state.
#[derive(Debug, Copy, Clone)]
pub struct Pipe {
    /// Data stored in the pipe.
    data: [u8; PIPESIZE],
    /// Number of bytes read.
    nread: u32,
    /// Number of bytes written.
    nwrite: u32,
    /// Read fd is still open.
    readopen: i32,
    /// Write fd is still open.
    writeopen: i32,
}
