//! \file sysproc.rs
//! \brief Rust implementations of simple system call handlers.
//!
//! All handlers wrap core kernel primitives exposed through other modules. Each
//! function mirrors the corresponding C implementation but leverages Rust's
//! safety features where feasible. The module exposes C ABI symbols so the
//! existing C kernel can invoke these handlers directly.
use crate::proc::{exit, fork, growproc, kill, myproc, sleep, wait};
use crate::syscall::argint;
use crate::trap::ticks;
use x86::io::outw;

use core::ffi::c_void;

/// \brief Create a child process.
///
/// Wraps the core `fork` routine exposed from the process module.
/// Returns the child's PID to the parent and 0 to the child or -1 on failure.
#[no_mangle]
pub unsafe extern "C" fn sys_fork() -> i32 {
    fork()
}

/// \brief Terminate the current process.
///
/// This call never returns to the caller. A return value of 0 merely
/// satisfies the C ABI expectations.
#[no_mangle]
pub unsafe extern "C" fn sys_exit() -> i32 {
    exit();
    0
}

/// \brief Send a kill signal to a process.
///
/// The PID is read from the first system call argument. If parsing fails,
/// `-1` is returned.
#[no_mangle]
pub unsafe extern "C" fn sys_kill() -> i32 {
    let mut pid: i32 = 0;
    if argint(0, &mut pid as *mut i32) < 0 {
        -1
    } else {
        kill(pid)
    }
}

/// \brief Adjust the process data segment size.
///
/// The increment in bytes is read from the first system call argument. The
/// previous segment size is returned on success or `-1` on failure.
#[no_mangle]
pub unsafe extern "C" fn sys_sbrk() -> i32 {
    let mut n: i32 = 0;
    if argint(0, &mut n as *mut i32) < 0 {
        return -1;
    }
    let addr = (*myproc()).sz;
    if growproc(n) < 0 {
        return -1;
    }
    addr as i32
}

/// \brief Sleep for a number of clock ticks.
///
/// The duration in ticks is provided as the first system call argument.
/// The process may be interrupted if it is killed during the sleep.
#[no_mangle]
pub unsafe extern "C" fn sys_sleep() -> i32 {
    let mut n: i32 = 0;
    if argint(0, &mut n as *mut i32) < 0 {
        return -1;
    }
    let start = ticks;
    while ((ticks - start) as i32) < n {
        if (*myproc()).killed != 0 {
            return -1;
        }
        sleep(&ticks as *const u32 as *const c_void, core::ptr::null());
    }
    0
}

/// \brief Wait for a child process to exit.
///
/// Returns the PID of the terminated child or `-1` on failure.
#[no_mangle]
pub unsafe extern "C" fn sys_wait() -> i32 {
    wait()
}

/// \brief Retrieve the current process identifier.
#[no_mangle]
pub unsafe extern "C" fn sys_getpid() -> i32 {
    (*myproc()).pid as i32
}

/// \brief Report the number of ticks since boot.
#[no_mangle]
pub unsafe extern "C" fn sys_uptime() -> i32 {
    ticks as i32
}

/// \brief Power off the machine via the QEMU "isa-debug-exit" port.
#[no_mangle]
pub unsafe extern "C" fn sys_halt() -> i32 {
    outw(0x604, 0x0 | 0x2000);
    0
}
