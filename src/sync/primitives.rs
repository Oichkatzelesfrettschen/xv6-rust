//! Basic synchronization primitives.

use core::sync::atomic::{AtomicU32, Ordering};
use crate::cpu_features; // For has_cmpxchg8b

pub struct SpinLock {
    locked: AtomicU32,
}

impl SpinLock {
    pub const fn new() -> Self {
        Self { locked: AtomicU32::new(0) }
    }

    pub fn try_lock(&self) -> bool {
        if cpu_features::has_cmpxchg8b() { // Assuming cmpxchg8b implies working 32-bit CAS
            self.locked.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok()
        } else {
            // Fallback for systems without cmpxchg8b (e.g., very old i386 before P5 with cx8)
            // True bare-metal interrupt disable/enable is complex and platform specific.
            // This placeholder indicates where such logic would go.
            // For xv6 on i386, cmpxchg8b should generally be available if we target P5+.
            // Our i386.json has +cx8, so this path is less likely unless on very old CPU.
            // todo!("Implement interrupt disable/enable or alternative non-CAS lock mechanism for try_lock");
            // For compilation, let's return false for now in the todo path.
            false
        }
    }

    pub fn lock(&self) {
        // Simple spin loop
        while !self.try_lock() {
            core::hint::spin_loop();
        }
    }

    pub fn unlock(&self) {
        self.locked.store(0, Ordering::Release);
    }
}
