use crate::proc::Cpu;

use core::ffi;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// \brief Minimal spinlock structure mirroring the C implementation.
pub struct Spinlock {
    locked: u32,
    name: *const u8,
    cpu: *const Cpu,
    pcs: [u32; 10],
}

extern "C" {
    /// \brief Acquire the lock, spinning until available.
    pub fn acquire(s: *const Spinlock);
    /// \brief Release the previously held lock.
    pub fn release(s: *const Spinlock);
    /// \brief Record call stack into `pcs` for debugging.
    pub fn getcallerpcs(v: ffi::c_void, pcs: *const u32);
}
