#![allow(dead_code)]

use crate::cpu_features::{has_sse2, has_sse4_2}; // Note: has_sse4_2 needs to be added
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
    if s.is_empty() { return 0; }

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
        if s1[i] < s2[i] { return -1; }
        if s1[i] > s2[i] { return 1; }
    }
    if len1 < len2 { -1 }
    else if len1 > len2 { 1 }
    else { 0 }
}

#[cfg(target_feature = "sse4.2")]
#[target_feature(enable = "sse4.2")]
#[inline]
unsafe fn strcmp_sse42(s1: &[u8], s2: &[u8]) -> i32 {
    let len1 = s1.len();
    let len2 = s2.len();

    let ptr1 = s1.as_ptr() as *const __m128i;
    let ptr2 = s2.as_ptr() as *const __m128i;

    // Mode for pcmpistri: null-terminated bytes, equal ordered, signed bytes
    const MODE: i32 = 0b0000_1100;

    let mut result_idx: i32;
    let mut s1_processed_len = 0;
    let mut s2_processed_len = 0;

    // This loop handles strings longer than 16 bytes by comparing 16-byte chunks
    // This is a simplified loop; real strcmp needs to handle nulls within chunks carefully.
    // PCMPISTRI can find the first null implicitly if lengths are > 15.
    loop {
        let l1 = core::cmp::min(len1 - s1_processed_len, 16);
        let l2 = core::cmp::min(len2 - s2_processed_len, 16);

        if l1 == 0 || l2 == 0 { // One string (or both) exhausted
            break;
        }

        let xmm1 = _mm_loadu_si128(ptr1.add(s1_processed_len / 16));
        let xmm2 = _mm_loadu_si128(ptr2.add(s2_processed_len / 16));

        // _SIDD_CMP_EQUAL_ORDERED implicitly uses string lengths if la/lb in rax/rdx,
        // or searches for nulls if lengths are >=16.
        // For explicit length substrings, other modes are better, or manual null padding.
        // This is a complex instruction. For now, let's assume it finds first difference.
        result_idx = _mm_cmpistri(xmm1, xmm2, MODE);
        let result_zf = _mm_cmpistrs(xmm1, xmm2, MODE); // Check ZF for equality up to min_len_chunk

        if result_idx < core::cmp::min(l1,l2) { // Mismatch found within current 16-byte chunk
            let char1 = s1[s1_processed_len + result_idx as usize];
            let char2 = s2[s2_processed_len + result_idx as usize];
            return (char1 as i32) - (char2 as i32);
        }

        // If ZF is set by _mm_cmpistrs, it means equal up to min(len(xmm1), len(xmm2))
        // This doesn't directly give strcmp result if one string is prefix of another AND null terminated.
        // This simplified loop isn't a full strcmp.
        if result_idx < 16 { // Found a null or difference in one of the strings
             break; // Will be handled by scalar comparison of remaining or length diff
        }

        s1_processed_len += 16;
        s2_processed_len += 16;

        if s1_processed_len >= len1 || s2_processed_len >= len2 {
            break;
        }
    }
    // Fallback to scalar for remaining parts or if one string is a prefix of another.
    // This SSE4.2 version is incomplete for a full strcmp.
    strcmp_scalar(s1, s2)
}

#[inline(always)]
pub fn strcmp_fast_slice(s1: &[u8], s2: &[u8]) -> i32 {
    // #[cfg(target_feature = "sse4.2")] // Temporarily disable until fully correct
    // if has_sse4_2() {
    //     return unsafe { strcmp_sse42(s1, s2) };
    // }
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
    if haystack.is_empty() { return None; }

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
    if haystack.is_empty() { return 0; }

    #[cfg(target_feature = "sse2")]
    if has_sse2() {
        return unsafe { count_bytes_sse2(haystack, needle) };
    }
    count_bytes_scalar(haystack, needle)
}
