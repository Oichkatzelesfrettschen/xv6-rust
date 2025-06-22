#![no_std]
#![feature(portable_simd)]
#![feature(thread_local)]
#![feature(c_size_t)]
//! \file lib.rs
//! \brief Core kernel crate exposing C ABI entrypoints.

// Module uses rely on explicit macro imports in each file

pub mod arch;
#[macro_use]
pub mod console;
pub mod allocator;
pub mod cpu_features;
pub mod file;
pub mod fpu_state;
pub mod fs;
pub mod ioapic;
pub mod kbd;
pub mod lapic;
pub mod mmu;
pub mod param;
pub mod pipe;
pub mod proc;
pub mod simd_integration;
pub mod simd_mem;
pub mod simd_string;
pub mod spinlock;
pub mod string;
pub mod sync;
pub mod syscall;
pub mod sysproc;
pub mod trap;
pub mod traps;
pub mod types;
pub mod uart;

use core::panic::PanicInfo;

/// \brief Kernel entry point once memory and CPUs are initialized.
///
/// This function is invoked from the C portion of the boot process and
/// prints a greeting to confirm Rust has been reached.
#[no_mangle]
pub unsafe extern "C" fn kmain() {
    cpu_features::init();
    println!("Hello from {}", "Rust");
}

/// \brief Minimal panic handler used during early bring-up.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
