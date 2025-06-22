#![allow(dead_code)]

use crate::cpu_features::{has_sse2, has_sse4_2};
use core::arch::asm;
// Ensure core::arch::x86 is available for intrinsics
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

// --- strlen ---
#[inline(always)]
fn strlen_scalar(s: &[u8]) -> usize {
    let mut i = 0;
    while i < s.len() && s[i] != 0 {
        i += 1;
    }
    i
}

#[cfg(target_feature = "sse2")]
#[target_feature(enable = "sse2")]
#[inline]
unsafe fn strlen_sse2(s: &[u8]) -> usize {
    let ptr = s.as_ptr();
    let len = s.len();
    let mut offset = 0;

    let zero_xmm = _mm_setzero_si128();

    while offset + 16 <= len {
        let current_ptr = ptr.add(offset);
        let data_xmm = _mm_loadu_si128(current_ptr as *const __m128i);
        let cmp_res_xmm = _mm_cmpeq_epi8(data_xmm, zero_xmm);
        let mask = _mm_movemask_epi8(cmp_res_xmm) as u16;

        if mask != 0 {
            return offset + mask.trailing_zeros() as usize;
        }
        offset += 16;
    }

    if offset < len {
        offset += strlen_scalar(&s[offset..]);
    }
    offset
}

#[inline(always)]
pub fn strlen_fast_slice(s: &[u8]) -> usize {
    if s.is_empty() {
        return 0;
    }

    #[cfg(target_feature = "sse2")]
    if has_sse2() {
        return unsafe { strlen_sse2(s) };
    }
    strlen_scalar(s)
}

// --- strcmp ---
#[inline(always)]
fn strcmp_scalar(s1: &[u8], s2: &[u8]) -> i32 {
    let len1 = strlen_scalar(s1);
    let len2 = strlen_scalar(s2);
    let min_len = core::cmp::min(len1, len2);

    for i in 0..min_len {
        if s1[i] < s2[i] {
            return -1;
        }
        if s1[i] > s2[i] {
            return 1;
        }
    }
    if len1 < len2 {
        -1
    } else if len1 > len2 {
        1
    } else {
        0
    }
}

#[cfg(target_feature = "sse4.2")]
#[target_feature(enable = "sse4.2")]
#[inline]
/// \brief Compare two strings using SSE4.2 instructions.
///
/// This implementation leverages `_mm_cmpistri` to efficiently locate the first
/// differing byte between the two inputs. It falls back to scalar comparison if
/// a difference occurs beyond the processed blocks.
///
/// \param s1 First byte slice to compare.
/// \param s2 Second byte slice to compare.
/// \return Negative if `s1 < s2`, positive if `s1 > s2`, and `0` if equal.
unsafe fn strcmp_sse42(s1: &[u8], s2: &[u8]) -> i32 {
    use core::arch::x86_64::*;
    const MODE: i32 =
        _SIDD_UBYTE_OPS | _SIDD_CMP_EQUAL_EACH | _SIDD_NEGATIVE_POLARITY | _SIDD_LEAST_SIGNIFICANT;
    let len1 = strlen_scalar(s1);
    let len2 = strlen_scalar(s2);
    let mut offset = 0;
    while offset < len1 && offset < len2 {
        let a = _mm_loadu_si128(s1.as_ptr().add(offset) as *const __m128i);
        let b = _mm_loadu_si128(s2.as_ptr().add(offset) as *const __m128i);
        let idx = _mm_cmpistri::<{ MODE }>(a, b);
        if idx < 16 {
            let i = offset + idx as usize;
            if i >= len1 || i >= len2 {
                break;
            }
            let c1 = s1[i];
            let c2 = s2[i];
            return (c1 as i32) - (c2 as i32);
        }
        offset += 16;
    }
    if len1 < len2 {
        -1
    } else if len1 > len2 {
        1
    } else {
        0
    }
}

/// \brief Optimized `strcmp` dispatcher.
///
/// Uses the SSE4.2 implementation when available, otherwise falls back to a
/// scalar byte-wise comparison.
#[inline(always)]
pub fn strcmp_fast_slice(s1: &[u8], s2: &[u8]) -> i32 {
    #[cfg(target_feature = "sse4.2")]
    if has_sse4_2() {
        return unsafe { strcmp_sse42(s1, s2) };
    }
    strcmp_scalar(s1, s2)
}

// --- memchr ---
#[inline(always)]
fn memchr_scalar(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

#[cfg(target_feature = "sse2")]
#[target_feature(enable = "sse2")]
#[inline]
unsafe fn memchr_sse2(haystack: &[u8], needle: u8) -> Option<usize> {
    let ptr = haystack.as_ptr();
    let len = haystack.len();
    let mut offset = 0;

    use core::arch::x86::*;
    let needle_xmm = _mm_set1_epi8(needle as i8);

    while offset + 16 <= len {
        let current_ptr = ptr.add(offset);
        let data_xmm = _mm_loadu_si128(current_ptr as *const __m128i);
        let cmp_res_xmm = _mm_cmpeq_epi8(data_xmm, needle_xmm);
        let mask = _mm_movemask_epi8(cmp_res_xmm) as u16;

        if mask != 0 {
            return Some(offset + mask.trailing_zeros() as usize);
        }
        offset += 16;
    }

    if offset < len {
        if let Some(pos) = memchr_scalar(&haystack[offset..], needle) {
            return Some(offset + pos);
        }
    }
    None
}

#[inline(always)]
pub fn memchr_fast_slice(haystack: &[u8], needle: u8) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }

    #[cfg(target_feature = "sse2")]
    if has_sse2() {
        return unsafe { memchr_sse2(haystack, needle) };
    }
    memchr_scalar(haystack, needle)
}

// --- count_bytes ---
#[inline(always)]
fn count_bytes_scalar(haystack: &[u8], needle: u8) -> usize {
    haystack.iter().filter(|&&b| b == needle).count()
}

#[cfg(target_feature = "sse2")]
#[target_feature(enable = "sse2")]
#[inline]
unsafe fn count_bytes_sse2(haystack: &[u8], needle: u8) -> usize {
    let ptr = haystack.as_ptr();
    let len = haystack.len();
    let mut offset = 0;
    let mut count = 0;

    use core::arch::x86::*;
    let needle_xmm = _mm_set1_epi8(needle as i8);

    while offset + 16 <= len {
        let current_ptr = ptr.add(offset);
        let data_xmm = _mm_loadu_si128(current_ptr as *const __m128i);
        let cmp_res_xmm = _mm_cmpeq_epi8(data_xmm, needle_xmm);
        let mask = _mm_movemask_epi8(cmp_res_xmm) as u16;
        count += mask.count_ones() as usize; // count_ones() on u16
        offset += 16;
    }

    if offset < len {
        count += count_bytes_scalar(&haystack[offset..], needle);
    }
    count
}

#[inline(always)]
pub fn count_bytes_fast_slice(haystack: &[u8], needle: u8) -> usize {
    if haystack.is_empty() {
        return 0;
    }

    #[cfg(target_feature = "sse2")]
    if has_sse2() {
        return unsafe { count_bytes_sse2(haystack, needle) };
    }
    count_bytes_scalar(haystack, needle)
}
