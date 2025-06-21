//! Kernel global allocator and heap initialization.

#![cfg_attr(feature = "alloc_debug", allow(dead_code, unused_imports, unused_variables))]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, NonNull};
use spin::Mutex; // Used by linked_list_allocator's LockedHeap directly.

use cfg_if::cfg_if;

// Import chosen allocator crate based on features
#[cfg(feature = "alloc_linked_list")]
use linked_list_allocator;

#[cfg(feature = "alloc_buddy_system")]
use buddy_system_allocator;


// --- Logging for Allocator (Debug) ---
// A simple logging macro that uses cprintf if available, or does nothing.
// This requires `simd_debug_print` feature to be enabled for cprintf to be linked.
// And `alloc_debug` to enable these log messages.
#[cfg(all(feature = "alloc_debug", feature = "simd_debug_print"))]
extern "C" {
    fn cprintf(fmt: *const u8, ...) -> core::ffi::c_int;
}

#[cfg(all(feature = "alloc_debug", feature = "simd_debug_print"))]
macro_rules! log {
    ($($arg:tt)*) => {{
        let fmt_str = concat!("[ALLOC] ", $($arg)*, "\n\0"); // Ensure newline and null term
        // unsafe { cprintf(fmt_str.as_ptr()); }
        crate::println!("{}", fmt_str); // Use Rust's println! macro, requires console support
    }};
}

#[cfg(not(all(feature = "alloc_debug", feature = "simd_debug_print")))]
macro_rules! log {
    ($($arg:tt)*) => {{}};
}


// --- Allocator Type Alias and Static Instance ---
cfg_if! {
    if #[cfg(feature = "alloc_linked_list")] {
        // linked_list_allocator::LockedHeap uses spin::Mutex internally.
        #[global_allocator]
        static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

    } else if #[cfg(feature = "alloc_buddy_system")] {
        // For buddy_system_allocator v0.9.1, with 'use_spin' feature,
        // its LockedHeap takes only the ORDER. The spinlock type is internal.
        #[global_allocator]
        static ALLOCATOR: buddy_system_allocator::LockedHeap<32> =
            buddy_system_allocator::LockedHeap::<32>::new();

    } else {
        // Fallback/Dummy allocator if no feature is selected
        struct DummyAllocator;
        unsafe impl GlobalAlloc for DummyAllocator {
            unsafe fn alloc(&self, _layout: Layout) -> *mut u8 { ptr::null_mut() }
            unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
                // Optional: panic here if you want to ensure it's never used.
                // panic!("dummy allocator dealloc should not be called");
            }
        }
        #[global_allocator]
        static ALLOCATOR: DummyAllocator = DummyAllocator;
    }
}


// --- Heap Initialization ---
/// Initializes the kernel heap. This function provides the memory region
/// that will be used by the selected Rust global allocator.
///
/// # Safety
/// - This function must be called only once during kernel initialization.
/// - `heap_start` must point to a valid, writable memory region.
/// - The memory region from `heap_start` to `heap_start + heap_size` must be
///   exclusively available for this heap and not used by other parts of the kernel
///   (e.g., C-side `kalloc` or page allocator) after this call, unless through
///   this allocator.
/// - `heap_start` should be aligned to a suitable value. Both `linked_list_allocator`
///   and `buddy_system_allocator` work well with page-aligned regions.
///   A minimum of 16-byte alignment is recommended for SSE compatibility if
///   allocated data might be used with SSE instructions.
/// - The `heap_size` must be large enough to accommodate allocator overhead and
///   actual allocations. A few KiB is a typical minimum.
///
/// # Current Integration
/// Currently, this function is called from Rust's `kmain` with placeholder values
/// `(0, 0)`. For functional heap allocation, this function **must** be called
/// from the C side of xv6 (e.g., in `main.c` after `kinit()`) with a valid
/// memory region.
///
/// # C-Side Invocation Example (Conceptual)
/// ```c
/// // In C (e.g., main.c after kernel memory is initialized):
/// extern void init_rust_heap(size_t heap_start, size_t heap_size);
/// // ...
/// // char* rust_heap_region = kalloc_some_region(RUST_HEAP_SIZE); // Or take from free list
/// // init_rust_heap((size_t)rust_heap_region, RUST_HEAP_SIZE);
/// ```
#[no_mangle]
pub unsafe extern "C" fn init_rust_heap(heap_start: usize, heap_size: usize) {
    log!("Initializing Rust heap. Start: {:#x}, Size: {:#x}", heap_start, heap_size);

    cfg_if! {
        if #[cfg(feature = "alloc_linked_list")] {
            ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
            log!("linked_list_allocator initialized.");
        } else if #[cfg(feature = "alloc_buddy_system")] {
            // buddy_system_allocator LockedHeap::init takes (bottom: usize, size: usize)
            ALLOCATOR.lock().init(heap_start, heap_size);
            log!("buddy_system_allocator initialized.");
        } else {
            if heap_start != 0 || heap_size != 0 {
                 log!("Warning: No specific Rust allocator feature enabled (using dummy), but init_rust_heap called with non-zero values.");
            }
        }
    }
}


// --- Heap Stats (Optional) ---
#[no_mangle]
pub unsafe extern "C" fn rust_heap_stats() {
    log!("--- Rust Heap Stats ---");
    cfg_if! {
        if #[cfg(feature = "alloc_linked_list")] {
            let allocator_guard = ALLOCATOR.lock();
            log!(" Used: {} bytes", allocator_guard.used());
            log!(" Free: {} bytes", allocator_guard.free());
        } else if #[cfg(feature = "alloc_buddy_system")] {
            let _allocator_guard = ALLOCATOR.lock(); // Prefixed with underscore
            // Corrected method names for buddy_system_allocator v0.9.1
            log!(" Used: {} bytes", _allocator_guard.stats_alloc_actual());
            log!(" Free: {} bytes", _allocator_guard.stats_free());
            log!(" Total: {} bytes", _allocator_guard.stats_total());
        } else {
            log!(" No specific allocator enabled for stats.");
        }
    }
    log!("----------------------");
}


// --- Allocation Error Handler ---
#[cfg(feature = "oom_panic_handler")]
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    log!("ALLOCATION ERROR: layout = {:?}", layout);
    panic!("allocation error: {:?}", layout)
}

// Helper for NonNull::new, primarily for alloc_debug usage if needed.
// Not strictly required by the allocator itself.
#[cfg(feature = "alloc_debug")]
fn new_non_null<T>(ptr: *mut T) -> Option<NonNull<T>> {
    NonNull::new(ptr)
}
