#![allow(dead_code)] // Allow unused functions for now

use crate::cpu_features::{has_mmx, has_sse2};
use core::arch::asm;

// Helper to repeat a byte N times into a u64 for MMX/SSE.
#[inline(always)]
const fn repeat_byte(byte: u8, times: usize) -> u64 {
    let mut val: u64 = 0;
    let mut i = 0;
    while i < times {
        val = (val << 8) | (byte as u64);
        i += 1;
    }
    val
}


// --- memcpy ---

#[inline(always)]
unsafe fn memcpy_scalar(dst: *mut u8, src: *const u8, len: usize) {
    core::ptr::copy_nonoverlapping(src, dst, len);
}

// MMX version is a scalar fallback due to "mmx" not being a valid target_feature / cfg.
#[inline]
unsafe fn memcpy_mmx(dst: *mut u8, src: *const u8, len: usize) {
    memcpy_scalar(dst, src, len);
    // No EMMS needed as no MMX instructions are used.
}


#[cfg(target_feature = "sse2")]
#[target_feature(enable = "sse2")]
#[inline]
unsafe fn memcpy_sse2(dst: *mut u8, src: *const u8, len: usize) {
    let mut d: *mut u8 = dst;
    let mut s: *const u8 = src;
    let mut n: usize = len;

    // Copy 16-byte blocks using SSE2 (XMM registers)
    while n >= 16 {
        asm!(
            "movups xmm0, [{s}]",  // Unaligned load
            "movups [{d}], xmm0",  // Unaligned store
            s = in(reg) s,
            d = in(reg) d,
            out("xmm0") _, // xmm0 is clobbered
            options(nostack, preserves_flags)
        );
        s = s.add(16);
        d = d.add(16);
        n -= 16;
    }

    // Remainder (0-15 bytes) handled by scalar copy.
    // MMX remainder attempt removed as MMX functions are scalar fallbacks.
    if n > 0 {
        memcpy_scalar(d, s, n);
    }
}


#[inline(always)]
pub unsafe fn memcpy_fast(dst: *mut u8, src: *const u8, len: usize) {
    if len == 0 { return; }

    #[cfg(target_feature = "sse2")]
    if has_sse2() {
        memcpy_sse2(dst, src, len);
        return;
    }

    // MMX path uses scalar fallback
    if has_mmx() {
        memcpy_mmx(dst, src, len);
        return;
    }

    memcpy_scalar(dst, src, len);
}


// --- memset ---

#[inline(always)]
unsafe fn memset_scalar(dst: *mut u8, val: u8, len: usize) {
    core::ptr::write_bytes(dst, val, len);
}


// MMX version is a scalar fallback.
#[inline]
unsafe fn memset_mmx(dst: *mut u8, val: u8, len: usize) {
    memset_scalar(dst, val, len);
    // No EMMS needed.
}


#[cfg(target_feature = "sse2")]
#[target_feature(enable = "sse2")]
#[inline]
unsafe fn memset_sse2(dst: *mut u8, val: u8, len: usize) {
    let mut d: *mut u8 = dst;
    let mut n: usize = len;

    if n >= 16 {
        let xmm_val_bytes = [val; 16];
        let xmm_val_ptr = xmm_val_bytes.as_ptr();
        asm!(
            "movups xmm0, [{val_ptr}]",
            val_ptr = in(reg) xmm_val_ptr,
            out("xmm0") _,
            options(nostack, preserves_flags, nomem)
        );
        while n >= 16 {
            asm!(
                "movups [{d}], xmm0",
                d = in(reg) d,
                options(nostack, preserves_flags) // xmm0 is implicit input
            );
            d = d.add(16);
            n -= 16;
        }
    }

    // MMX remainder attempt removed.
    if n > 0 {
        memset_scalar(d, val, n);
    }
}


#[inline(always)]
pub unsafe fn memset_fast(dst: *mut u8, val: u8, len: usize) {
    if len == 0 { return; }

    #[cfg(target_feature = "sse2")]
    if has_sse2() {
        memset_sse2(dst, val, len);
        return;
    }

    // MMX path uses scalar fallback
    if has_mmx() {
        memset_mmx(dst, val, len);
        return;
    }

    memset_scalar(dst, val, len);
}
