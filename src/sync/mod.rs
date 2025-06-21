// src/sync/mod.rs
pub mod primitives;
pub mod ticket_lock; // Renamed module

pub use primitives::SpinLock;
pub use ticket_lock::{TicketLock, TicketLockGuard}; // Updated exports
