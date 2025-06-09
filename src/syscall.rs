//! \file syscall.rs
//! \brief Rust declarations for C system call helpers.

extern "C" {
    /// \brief Fetch an integer argument from the system call.
    pub fn argint(n: i32, ip: *mut i32) -> i32;
}
