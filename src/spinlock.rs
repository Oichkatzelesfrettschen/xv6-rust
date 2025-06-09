//! \file spinlock.rs
//! \brief Spinlock mechanism for mutual exclusion.

use crate::proc::Cpu;
use core::ffi;

/// \brief Spinlock structure used for mutual exclusion.
/// 
/// Mirrors the C implementation for interoperability.
///
/// Fields:
/// - `locked`: Indicates if the lock is held (`1`) or free (`0`).
/// - `name`: Pointer to a null-terminated name string (for debugging).
/// - `cpu`: Pointer to the CPU holding the lock when locked.
/// - `pcs`: Call stack program counters captured at lock acquisition.
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Spinlock {
    /// \brief Indicates lock state: `0` == unlocked, `1` == locked.
    pub locked: u32,
    /// \brief Name of the lock (null-terminated C string).
    pub name:   *const u8,
    /// \brief CPU currently holding the lock.
    pub cpu:    *const Cpu,
    /// \brief Call stack PCs for debugging (captured on acquire).
    pub pcs:    [u32; 10],
}

extern "C" {
    /// \brief Acquire the spinlock, spinning until it becomes available.
    ///
    /// \param s Pointer to the spinlock to acquire.
    pub fn acquire(s: *const Spinlock);

    /// \brief Release a previously acquired spinlock.
    ///
    /// \param s Pointer to the spinlock to release.
    pub fn release(s: *const Spinlock);

    /// \brief Capture the caller’s program counters into `pcs`.
    ///
    /// Used internally to record the call stack when acquiring the lock.
    ///
    /// \param v Unused placeholder for ABI compatibility.
    /// \param pcs Pointer to an array of `u32` where PCs will be stored.
    pub fn getcallerpcs(v: ffi::c_void, pcs: *const u32);
}
