// #![no_std] // Removed: crate-level attribute should be in the root module

use core::cell::UnsafeCell;
// use core::fmt; // Removed: unused
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU32, Ordering}; // Changed AtomicU64 to AtomicU32

// QuaternionTickets struct removed as it was unused.

// AtomicQuaternion now holds two AtomicU32s.
#[derive(Debug)] // Default cannot be easily derived for AtomicU32 without const fn new.
struct AtomicTicketLockState {
    next_ticket: AtomicU32,
    current_ticket: AtomicU32,
}

impl AtomicTicketLockState {
    const fn new(next: u32, current: u32) -> Self {
        Self {
            next_ticket: AtomicU32::new(next),
            current_ticket: AtomicU32::new(current),
        }
    }

    fn load_current_ticket(&self, order: Ordering) -> u32 {
        self.current_ticket.load(order)
    }

    // Atomically increment next_ticket and return the old ticket value (my_ticket)
    fn fetch_my_ticket(&self, order: Ordering) -> u32 {
        self.next_ticket.fetch_add(1, order)
    }

    // Atomically increment current_ticket (only called by the current lock holder)
    fn release_ticket(&self, order: Ordering) {
        // In a simple ticket lock, current_ticket is incremented.
        // No complex CAS needed as only one thread (holder) does this.
        // However, fetch_add is atomic and fine.
        self.current_ticket.fetch_add(1, order);
        // A stronger ordering might be needed if other cores must see this immediately
        // without further fencing, but Release is typical for unlocks.
    }
}

// --- Ticket Lock --- (Renamed from QuaternionSpinlock)
#[derive(Debug)]
pub struct TicketLock<T: ?Sized> {
    state: AtomicTicketLockState,
    data: UnsafeCell<T>,
}

impl<T: ?Sized + Default> Default for TicketLock<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

unsafe impl<T: ?Sized + Send> Send for TicketLock<T> {}
unsafe impl<T: ?Sized + Send> Sync for TicketLock<T> {}

impl<T> TicketLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicTicketLockState::new(0, 0), // Initial tickets
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> TicketLock<T> {
    pub fn lock(&self) -> TicketLockGuard<'_, T> {
        let my_ticket = self.state.fetch_my_ticket(Ordering::Relaxed);
        while self.state.load_current_ticket(Ordering::Acquire) != my_ticket {
            core::hint::spin_loop();
        }
        TicketLockGuard { lock: self }
    }

    // pub fn try_lock(&self) -> Option<TicketLockGuard<'_, T>> { ... }

    /// Safety: Called by TicketLockGuard::drop
    fn unlock_internal(&self) {
        self.state.release_ticket(Ordering::Release);
    }
}

// --- Ticket Lock Guard --- (Renamed from QuaternionGuard)
pub struct TicketLockGuard<'a, T: ?Sized> {
    lock: &'a TicketLock<T>,
}

impl<T: ?Sized> Deref for TicketLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for TicketLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for TicketLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock_internal();
    }
}

// Test module removed as per subtask instructions
