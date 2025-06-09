use crate::proc::Cpu;

use core::ffi;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
/// Spinlock structure used for mutual exclusion.
pub struct Spinlock {
    /// Is the lock held?
    locked: u32,
    /// Name for debugging.
    name: *const u8,
    /// CPU holding the lock.
    cpu: *const Cpu,
    /// Call stack that locked the spinlock.
    pcs: [u32; 10],
}

extern "C" {
    pub fn acquire(s: *const Spinlock);
    pub fn release(s: *const Spinlock);
    pub fn getcallerpcs(v: ffi::c_void, pcs: *const u32);
}
