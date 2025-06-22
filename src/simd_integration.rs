#![allow(dead_code)]

use super::fpu_state::{FpuManager, FpuState};
use super::simd_mem::{memcpy_fast, memset_fast};
use super::simd_string::{memchr_fast_slice, strcmp_fast_slice, strlen_fast_slice};
use core::ffi::{c_char, c_int, c_size_t as size_t, c_void};
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "simd_debug_print")]
extern "C" {
    fn cprintf(fmt: *const u8, ...) -> c_int;
}

#[cfg(feature = "simd_debug_print")]
macro_rules! debug_cprintf {
    ($($arg:tt)*) => {{
        let fmt_str = concat!($($arg)*, "\0"); // Null terminate for C
        // unsafe { cprintf(fmt_str.as_ptr()); } // Actual call commented out
    }};
}
#[cfg(not(feature = "simd_debug_print"))]
macro_rules! debug_cprintf {
    ($($arg:tt)*) => {{}};
}

#[thread_local]
static mut KERNEL_FPU_STATE: MaybeUninit<FpuManager> = MaybeUninit::uninit();
/// Indicates whether `KERNEL_FPU_STATE` has been initialized.
static SIMD_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// \brief Enter a critical section that allows use of SIMD/FPU instructions.
///
/// This function safely restores the kernel's FPU state if the subsystem has
/// been initialized via [`init_simd_subsystem`]. It becomes a no-op otherwise.
#[no_mangle]
pub unsafe extern "C" fn kernel_fpu_begin() {
    debug_cprintf!("kernel_fpu_begin\n");
    if SIMD_INITIALIZED.load(Ordering::Relaxed) {
        KERNEL_FPU_STATE.assume_init_mut().begin_use();
    }
}

/// \brief Exit the SIMD/FPU critical section.
///
/// If the FPU subsystem has not been initialized, this function simply
/// returns. Otherwise it saves the current FPU state for later use.
#[no_mangle]
pub unsafe extern "C" fn kernel_fpu_end() {
    debug_cprintf!("kernel_fpu_end\n");
    if SIMD_INITIALIZED.load(Ordering::Relaxed) {
        KERNEL_FPU_STATE.assume_init_mut().end_use();
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_memcpy(
    dst: *mut c_void,
    src: *const c_void,
    n: size_t,
) -> *mut c_void {
    debug_cprintf!("rust_memcpy called\n");
    memcpy_fast(dst as *mut u8, src as *const u8, n);
    dst
}

#[no_mangle]
pub unsafe extern "C" fn rust_memmove(
    dst: *mut c_void,
    src: *const c_void,
    n: size_t,
) -> *mut c_void {
    debug_cprintf!("rust_memmove called\n");
    core::ptr::copy(src as *const u8, dst as *mut u8, n);
    dst
}

#[no_mangle]
pub unsafe extern "C" fn rust_memset(dst: *mut c_void, c: c_int, n: size_t) -> *mut c_void {
    debug_cprintf!("rust_memset called\n");
    memset_fast(dst as *mut u8, c as u8, n);
    dst
}

unsafe fn c_strlen(s_ptr: *const c_char) -> usize {
    let mut len = 0;
    let mut current = s_ptr as *const u8;
    while *current != 0 {
        current = current.add(1);
        len += 1;
    }
    len
}

#[no_mangle]
pub unsafe extern "C" fn rust_strlen(s: *const c_char) -> size_t {
    debug_cprintf!("rust_strlen called\n");
    let len = c_strlen(s);
    let slice = core::slice::from_raw_parts(s as *const u8, len);
    strlen_fast_slice(slice)
}

#[no_mangle]
pub unsafe extern "C" fn rust_strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    debug_cprintf!("rust_strcmp called\n");
    let len1 = c_strlen(s1);
    let slice1 = core::slice::from_raw_parts(s1 as *const u8, len1);
    let len2 = c_strlen(s2);
    let slice2 = core::slice::from_raw_parts(s2 as *const u8, len2);
    strcmp_fast_slice(slice1, slice2)
}

#[no_mangle]
pub unsafe extern "C" fn rust_strncmp(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int {
    debug_cprintf!("rust_strncmp called\n");
    let mut i = 0;
    while i < n {
        let b1 = *(s1 as *const u8).add(i);
        let b2 = *(s2 as *const u8).add(i);
        if b1 < b2 {
            return -1;
        }
        if b1 > b2 {
            return 1;
        }
        if b1 == 0 {
            return 0;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn rust_strchr(s: *const c_char, c: c_int) -> *mut c_char {
    debug_cprintf!("rust_strchr called\n");
    let needle = c as u8;
    let len = c_strlen(s);
    let slice = core::slice::from_raw_parts(s as *const u8, len);
    match memchr_fast_slice(slice, needle) {
        Some(index) => (s as *mut u8).add(index) as *mut c_char,
        None => core::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_copy_page(dst: *mut u8, src: *const u8) {
    memcpy_fast(dst, src, 4096);
}

#[no_mangle]
pub unsafe extern "C" fn rust_zero_page(dst: *mut u8) {
    memset_fast(dst, 0, 4096);
}

#[inline(always)]
pub unsafe fn memcmp_fast(s1: *const u8, s2: *const u8, len: usize) -> i32 {
    let mut i = 0;
    while i < len {
        let b1 = *s1.add(i);
        let b2 = *s2.add(i);
        if b1 < b2 {
            return -1;
        }
        if b1 > b2 {
            return 1;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn rust_pages_equal(p1: *const u8, p2: *const u8) -> c_int {
    if memcmp_fast(p1, p2, 4096) == 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_ip_checksum(addr: *const u8, mut count: c_int) -> u16 {
    let mut sum: u32 = 0;
    let mut current_addr = addr;
    while count > 1 {
        sum += unsafe { *(current_addr as *const u16) } as u32;
        current_addr = unsafe { current_addr.add(2) };
        count -= 2;
    }
    if count > 0 {
        sum += unsafe { *current_addr } as u32;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !sum as u16
}

/// \brief Initialize SIMD and FPU support for the kernel.
///
/// This must be invoked before any calls to [`kernel_fpu_begin`] or
/// [`kernel_fpu_end`]. It configures the processor's FPU and marks the
/// thread-local state as ready for use.
#[no_mangle]
pub unsafe extern "C" fn init_simd_subsystem() {
    debug_cprintf!("init_simd_subsystem called\n");
    crate::fpu_state::init_fpu();
    KERNEL_FPU_STATE.write(FpuManager::new());
    SIMD_INITIALIZED.store(true, Ordering::Relaxed);
}
