use core::sync::atomic::{AtomicBool, Ordering};
use raw_cpuid::CpuId;

static HAS_SSE: AtomicBool = AtomicBool::new(false);
static HAS_SSE2: AtomicBool = AtomicBool::new(false);
static HAS_MMX: AtomicBool = AtomicBool::new(false);
// static HAS_3DNOW: AtomicBool = AtomicBool::new(false); // Commented out
static HAS_SSE3: AtomicBool = AtomicBool::new(false);
static HAS_SSE4_1: AtomicBool = AtomicBool::new(false);
// static HAS_SSE4A: AtomicBool = AtomicBool::new(false); // Commented out
static HAS_CMPXCHG8B: AtomicBool = AtomicBool::new(false);
static HAS_XSAVE: AtomicBool = AtomicBool::new(false);
static HAS_FXSR: AtomicBool = AtomicBool::new(false);
static HAS_SSE4_2: AtomicBool = AtomicBool::new(false);

pub fn init() {
    let cpuid = CpuId::new();

    if let Some(feature_info) = cpuid.get_feature_info() {
        HAS_SSE.store(feature_info.has_sse(), Ordering::Relaxed);
        HAS_SSE2.store(feature_info.has_sse2(), Ordering::Relaxed);
        HAS_MMX.store(feature_info.has_mmx(), Ordering::Relaxed);
        HAS_SSE3.store(feature_info.has_sse3(), Ordering::Relaxed);
        HAS_SSE4_1.store(feature_info.has_sse41(), Ordering::Relaxed); // Changed has_sse4_1 to has_sse41
        HAS_CMPXCHG8B.store(feature_info.has_cmpxchg8b(), Ordering::Relaxed);
        HAS_XSAVE.store(feature_info.has_xsave(), Ordering::Relaxed);
        HAS_FXSR.store(feature_info.has_fxsave_fxstor(), Ordering::Relaxed);
        HAS_SSE4_2.store(feature_info.has_sse42(), Ordering::Relaxed);
    } else {
        // This case might occur on very old CPUs or if CPUID is somehow disabled/problematic.
        // For xv6 context, we'd expect feature_info to be available.
        // Consider a panic or a clear log if console is available and this happens.
        // For now, atomics remain false.
        if console_is_ready() {
             crate::println!("CPUID: Standard Feature Info not available!");
        }
    }

    if let Some(extended_feature_info) = cpuid.get_extended_feature_info() {
        // HAS_3DNOW.store(extended_feature_info.has_3dnow(), Ordering::Relaxed); // Commented out
        // HAS_SSE4A.store(extended_feature_info.has_sse4a(), Ordering::Relaxed); // Commented out
        let _ = extended_feature_info; // This line ensures extended_feature_info is "used"
    } else {
        if console_is_ready() {
            // Only print if we were expecting to use these features and they are enabled above
            // crate::println!("CPUID: Extended Feature Info (for 3DNow!/SSE4a) not available.");
        }
    }

    if console_is_ready() {
        print_detected_features();
    }
}

pub fn has_sse() -> bool {
    HAS_SSE.load(Ordering::Relaxed)
}

pub fn has_sse2() -> bool {
    HAS_SSE2.load(Ordering::Relaxed)
}

pub fn has_mmx() -> bool {
    HAS_MMX.load(Ordering::Relaxed)
}

// pub fn has_3dnow() -> bool { // Commented out
//     HAS_3DNOW.load(Ordering::Relaxed)
// }

pub fn has_sse3() -> bool {
    HAS_SSE3.load(Ordering::Relaxed)
}

pub fn has_sse4_1() -> bool {
    HAS_SSE4_1.load(Ordering::Relaxed)
}

// pub fn has_sse4a() -> bool { // Commented out
//     HAS_SSE4A.load(Ordering::Relaxed)
// }

pub fn has_cmpxchg8b() -> bool {
    HAS_CMPXCHG8B.load(Ordering::Relaxed)
}

pub fn has_xsave() -> bool {
    HAS_XSAVE.load(Ordering::Relaxed)
}

pub fn has_fxsr() -> bool {
    HAS_FXSR.load(Ordering::Relaxed)
}

pub fn has_sse4_2() -> bool {
    HAS_SSE4_2.load(Ordering::Relaxed)
}

// Placeholder for console readiness
fn console_is_ready() -> bool { true }

fn print_detected_features() {
    crate::println!("CPU Features Detected (Runtime via raw-cpuid):");
    crate::println!("  MMX:  {}", has_mmx());
    crate::println!("  SSE:  {}", has_sse());
    crate::println!("  SSE2: {}", has_sse2());
    crate::println!("  SSE3: {}", has_sse3());
    crate::println!("  SSE4.1: {}", has_sse4_1());
    // crate::println!("  SSE4a: {}", has_sse4a()); // Commented out
    // crate::println!("  3DNow!: {}", has_3dnow()); // Commented out
    crate::println!("  CMPXCHG8B: {}", has_cmpxchg8b());
    crate::println!("  XSAVE: {}", has_xsave());
    crate::println!("  FXSR: {}", has_fxsr());
    crate::println!("  SSE4.2: {}", has_sse4_2());
}
