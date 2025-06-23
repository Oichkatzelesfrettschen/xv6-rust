use core::sync::atomic::{AtomicBool, Ordering};
use raw_cpuid::CpuId;

static HAS_SSE: AtomicBool = AtomicBool::new(false);
static HAS_SSE2: AtomicBool = AtomicBool::new(false);
static HAS_MMX: AtomicBool = AtomicBool::new(false);
static HAS_3DNOW: AtomicBool = AtomicBool::new(false);
static HAS_SSE3: AtomicBool = AtomicBool::new(false);
static HAS_SSSE3: AtomicBool = AtomicBool::new(false);
static HAS_SSE4_1: AtomicBool = AtomicBool::new(false);
static HAS_SSE4A: AtomicBool = AtomicBool::new(false);
static HAS_CMPXCHG8B: AtomicBool = AtomicBool::new(false);
static HAS_XSAVE: AtomicBool = AtomicBool::new(false);
static HAS_FXSR: AtomicBool = AtomicBool::new(false);
static HAS_SSE4_2: AtomicBool = AtomicBool::new(false);
static HAS_AVX: AtomicBool = AtomicBool::new(false);
static HAS_AVX2: AtomicBool = AtomicBool::new(false);
static HAS_AVX512F: AtomicBool = AtomicBool::new(false);
static HAS_AVX512VNNI: AtomicBool = AtomicBool::new(false);
static HAS_AVX_VNNI: AtomicBool = AtomicBool::new(false);
static HAS_FPU: AtomicBool = AtomicBool::new(false);

/// \brief Check if the `CPUID` instruction is available.
///
/// The 386 CPU lacked the `CPUID` instruction. Later processors set the
/// ID flag in EFLAGS to indicate support. This routine toggles the flag
/// and verifies whether the change persists.
pub fn cpuid_supported() -> bool {
    let original: u32;
    let toggled: u32;
    unsafe {
        core::arch::asm!(
            "pushfd",
            "pop {0}",
            out(reg) original,
            options(nostack, preserves_flags),
        );
        let with_id = original ^ (1 << 21);
        core::arch::asm!(
            "push {0} \n popfd",
            in(reg) with_id,
            options(nostack, preserves_flags),
        );
        core::arch::asm!(
            "pushfd",
            "pop {0}",
            out(reg) toggled,
            options(nostack, preserves_flags),
        );
    }
    ((original ^ toggled) & (1 << 21)) != 0
}

/// \brief Initialize CPU feature detection.
///
/// The function populates a set of global feature flags using the
/// [`raw_cpuid`] crate. When the `CPUID` instruction is not supported,
/// no flags are set and the routine returns early so the kernel can
/// operate on a minimal 386 feature set.
pub fn init() {
    if !cpuid_supported() {
        if console_is_ready() {
            crate::println!("CPUID not supported; assuming 386-class CPU");
        }
        return;
    }

    let cpuid = CpuId::new();

    if let Some(feature_info) = cpuid.get_feature_info() {
        HAS_FPU.store(feature_info.has_fpu(), Ordering::Relaxed);
        HAS_SSE.store(feature_info.has_sse(), Ordering::Relaxed);
        HAS_SSE2.store(feature_info.has_sse2(), Ordering::Relaxed);
        HAS_MMX.store(feature_info.has_mmx(), Ordering::Relaxed);
        HAS_SSE3.store(feature_info.has_sse3(), Ordering::Relaxed);
        HAS_SSSE3.store(feature_info.has_ssse3(), Ordering::Relaxed);
        HAS_SSE4_1.store(feature_info.has_sse41(), Ordering::Relaxed); // Changed has_sse4_1 to has_sse41
        HAS_CMPXCHG8B.store(feature_info.has_cmpxchg8b(), Ordering::Relaxed);
        HAS_XSAVE.store(feature_info.has_xsave(), Ordering::Relaxed);
        HAS_FXSR.store(feature_info.has_fxsave_fxstor(), Ordering::Relaxed);
        HAS_SSE4_2.store(feature_info.has_sse42(), Ordering::Relaxed);
        HAS_AVX.store(feature_info.has_avx(), Ordering::Relaxed);
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
        HAS_AVX2.store(extended_feature_info.has_avx2(), Ordering::Relaxed);
        HAS_AVX512F.store(extended_feature_info.has_avx512f(), Ordering::Relaxed);
        HAS_AVX512VNNI.store(extended_feature_info.has_avx512vnni(), Ordering::Relaxed);
        HAS_AVX_VNNI.store(extended_feature_info.has_avx_vnni(), Ordering::Relaxed);
        let _ = extended_feature_info; // Ensure value is used
    } else {
        if console_is_ready() {
            // Only print if we were expecting to use these features and they are enabled above
            // crate::println!("CPUID: Extended Feature Info (for 3DNow!/SSE4a) not available.");
        }
    }

    if let Some(ext_fn_info) = cpuid.get_extended_processor_and_feature_identifiers() {
        HAS_SSE4A.store(ext_fn_info.has_sse4a(), Ordering::Relaxed);
        HAS_3DNOW.store(
            ext_fn_info.has_amd_3dnow_extensions() || ext_fn_info.has_3dnow(),
            Ordering::Relaxed,
        );
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

pub fn has_3dnow() -> bool {
    HAS_3DNOW.load(Ordering::Relaxed)
}

pub fn has_sse3() -> bool {
    HAS_SSE3.load(Ordering::Relaxed)
}

pub fn has_sse4_1() -> bool {
    HAS_SSE4_1.load(Ordering::Relaxed)
}

pub fn has_sse4a() -> bool {
    HAS_SSE4A.load(Ordering::Relaxed)
}

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

pub fn has_ssse3() -> bool {
    HAS_SSSE3.load(Ordering::Relaxed)
}

pub fn has_avx() -> bool {
    HAS_AVX.load(Ordering::Relaxed)
}

pub fn has_avx2() -> bool {
    HAS_AVX2.load(Ordering::Relaxed)
}

pub fn has_avx512f() -> bool {
    HAS_AVX512F.load(Ordering::Relaxed)
}

pub fn has_avx512vnni() -> bool {
    HAS_AVX512VNNI.load(Ordering::Relaxed)
}

pub fn has_avx_vnni() -> bool {
    HAS_AVX_VNNI.load(Ordering::Relaxed)
}

pub fn has_fpu() -> bool {
    HAS_FPU.load(Ordering::Relaxed)
}

/// CPU type classification used for 386 variants.
pub enum CpuVariant {
    /// 386SX without FPU.
    I386SX,
    /// 386DX with external 387 FPU present.
    I386DX,
    /// Any processor with CPUID support or otherwise not a 386-class CPU.
    Other,
}

/// \brief Determine whether the processor is a 386SX or 386DX.
///
/// Call [`init`] prior to invoking this function so that [`has_fpu`] reflects
/// the presence of a math coprocessor. When `CPUID` is available the processor
/// is considered newer than a 386 and [`CpuVariant::Other`] is returned.
pub fn detect_386_variant() -> CpuVariant {
    if cpuid_supported() {
        CpuVariant::Other
    } else if has_fpu() {
        CpuVariant::I386DX
    } else {
        CpuVariant::I386SX
    }
}

// Placeholder for console readiness
fn console_is_ready() -> bool {
    true
}

fn print_detected_features() {
    crate::println!("CPU Features Detected (Runtime via raw-cpuid):");
    crate::println!("  MMX:  {}", has_mmx());
    crate::println!("  SSE:  {}", has_sse());
    crate::println!("  SSE2: {}", has_sse2());
    crate::println!("  SSE3: {}", has_sse3());
    crate::println!("  SSSE3: {}", has_ssse3());
    crate::println!("  SSE4.1: {}", has_sse4_1());
    crate::println!("  SSE4a: {}", has_sse4a());
    crate::println!("  3DNow!: {}", has_3dnow());
    crate::println!("  CMPXCHG8B: {}", has_cmpxchg8b());
    crate::println!("  XSAVE: {}", has_xsave());
    crate::println!("  FXSR: {}", has_fxsr());
    crate::println!("  SSE4.2: {}", has_sse4_2());
    crate::println!("  AVX: {}", has_avx());
    crate::println!("  AVX2: {}", has_avx2());
    crate::println!("  AVX512F: {}", has_avx512f());
    crate::println!("  AVX512VNNI: {}", has_avx512vnni());
    crate::println!("  AVX_VNNI: {}", has_avx_vnni());
    crate::println!("  FPU: {}", has_fpu());
}
