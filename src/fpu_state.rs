#![allow(dead_code)]

use crate::cpu_features::{has_fxsr, has_sse, has_xsave}; // Note: has_fxsr needs to be added
use bitfield::bitfield;
use core::arch::asm;

// --- FPU Status Word ---
bitfield! {
    #[derive(Copy, Clone, Default)]
    pub struct FpuStatusWord(u16);
    impl Debug;
    pub busy, set_busy: 15;
    pub condition_code_3, set_condition_code_3: 14;
    pub top_of_stack, set_top_of_stack: 13, 11;
    pub condition_code_2, set_condition_code_2: 10;
    pub condition_code_1, set_condition_code_1: 9;
    pub condition_code_0, set_condition_code_0: 8;
    pub error_summary_status, set_error_summary_status: 7;
    pub stack_fault, set_stack_fault: 6;
    pub precision_flag, set_precision_flag: 5;
    pub underflow_flag, set_underflow_flag: 4;
    pub overflow_flag, set_overflow_flag: 3;
    pub zero_divide_flag, set_zero_divide_flag: 2;
    pub denormalized_operand_flag, set_denormalized_operand_flag: 1;
    pub invalid_operation_flag, set_invalid_operation_flag: 0;
}

// --- MXCSR Register ---
bitfield! {
    #[derive(Copy, Clone, Default)]
    pub struct MxcsrRegister(u32);
    impl Debug;
    pub flush_to_zero, set_flush_to_zero: 15;
    pub rounding_control, set_rounding_control: 14, 13;
    pub precision_mask, set_precision_mask: 12;
    pub underflow_mask, set_underflow_mask: 11;
    pub overflow_mask, set_overflow_mask: 10;
    pub zero_divide_mask, set_zero_divide_mask: 9;
    pub denormal_mask, set_denormal_mask: 8;
    pub invalid_operation_mask, set_invalid_operation_mask: 7;
    pub denormals_are_zeros, set_denormals_are_zeros: 6; // Only if DAZ bit (CR0.DAZ) in EFER is set
    pub precision_flag_sticky, set_precision_flag_sticky: 5;
    pub underflow_flag_sticky, set_underflow_flag_sticky: 4;
    pub overflow_flag_sticky, set_overflow_flag_sticky: 3;
    pub zero_divide_flag_sticky, set_zero_divide_flag_sticky: 2;
    pub denormal_flag_sticky, set_denormal_flag_sticky: 1;
    pub invalid_operation_flag_sticky, set_invalid_operation_flag_sticky: 0;
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FpuStateFormat {
    FSAVE = 0,  // Basic x87, MMX. 108 bytes.
    FXSAVE = 1, // x87, MMX, SSE. 512 bytes, 16-byte aligned.
    XSAVE = 2,  // x87, MMX, SSE, AVX, etc. Variable size, 16-byte aligned (or 64 for AVX512).
}

#[repr(C, align(64))] // Align to 64 for XSAVE (AVX512 requires this)
pub struct FpuState {
    data: [u8; 512 + 64], // 512 for FXSAVE area, + space for XSAVE header/extended states if needed.
                          // Actual XSAVE size can be larger and determined by CPUID.
                          // For simplicity, we use a fixed-size buffer that's large enough for common cases.
    format: FpuStateFormat,
}

impl Default for FpuState {
    fn default() -> Self {
        Self {
            data: [0; 512 + 64],
            format: FpuStateFormat::FSAVE, // Default to basic format
        }
    }
}

impl FpuState {
    pub fn new() -> Self {
        let mut state: Self = Default::default();
        // Determine the best format supported by the CPU
        if has_xsave() {
            state.format = FpuStateFormat::XSAVE;
        } else if has_fxsr() { // FXSR is implied by SSE, but good to check explicitly
            state.format = FpuStateFormat::FXSAVE;
        } else {
            state.format = FpuStateFormat::FSAVE;
        }
        state
    }

    pub unsafe fn save(&mut self) {
        match self.format {
            FpuStateFormat::XSAVE => {
                // Simplified XSAVE: uses fxsave portion. XCR0 should be set to save relevant states.
                // To save all user states: xcr0 = 0x0000_0007 (x87, SSE, AVX) or more.
                // For now, we rely on fxsave behavior for compatibility.
                // A full XSAVE would query XCR0, use `xsave` or `xsaveopt`.
                asm!(
                    "fxsave [{}]", // Changed to fxsave for 32-bit target
                    in(reg) self.data.as_mut_ptr(),
                    options(nostack, preserves_flags),
                    clobber_abi("C")
                );
            }
            FpuStateFormat::FXSAVE => {
                asm!(
                    "fxsave [{}]", // Changed to fxsave
                    in(reg) self.data.as_mut_ptr(),
                    options(nostack, preserves_flags),
                    clobber_abi("C")
                );
            }
            FpuStateFormat::FSAVE => {
                asm!(
                    "fsave [{}]", // FSAVE clears FPU state
                    in(reg) self.data.as_mut_ptr(),
                    options(nostack, preserves_flags),
                    clobber_abi("C")
                );
            }
        }
    }

    pub unsafe fn restore(&self) {
        match self.format {
            FpuStateFormat::XSAVE => {
                // Simplified XRSTOR: uses fxrstor portion.
                // A full XRSTOR would query XCR0.
                asm!(
                    "fxrstor [{}]", // Changed to fxrstor
                    in(reg) self.data.as_ptr(),
                    options(nostack, preserves_flags),
                    clobber_abi("C")
                );
            }
            FpuStateFormat::FXSAVE => {
                asm!(
                    "fxrstor [{}]", // Changed to fxrstor
                    in(reg) self.data.as_ptr(),
                    options(nostack, preserves_flags),
                    clobber_abi("C")
                );
            }
            FpuStateFormat::FSAVE => {
                asm!(
                    "frstor [{}]",
                    in(reg) self.data.as_ptr(),
                    options(nostack, preserves_flags),
                    clobber_abi("C")
                );
            }
        }
    }

    pub fn is_xsave_format(&self) -> bool {
        self.format == FpuStateFormat::XSAVE
    }
}


/// Initializes the FPU.
/// For x86, this typically involves sending FINIT/FNINIT.
/// Also sets up CR0 and CR4 flags for FPU/SIMD operation.
pub unsafe fn init_fpu() {
    // 1. Enable FPU in CR0:
    //    Clear EM (bit 2) to indicate FPU is present.
    //    Set MP (bit 1) to enable FPU error reporting via INT 16 (for protected mode).
    //    Ensure NE (bit 5) is set for native FPU error handling (INT 16 / #MF).
    let mut cr0: u32;
    asm!("mov {0}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
    cr0 &= !(1 << 2); // Clear EM (Emulation)
    cr0 |= 1 << 1;  // Set MP (Monitor Coprocessor)
    cr0 |= 1 << 5;  // Set NE (Numeric Error)
    asm!("mov cr0, {0}", in(reg) cr0, options(nomem, nostack, preserves_flags));

    // 2. Enable SSE/SIMD exceptions in CR4:
    //    Set OSFXSR (bit 9) to enable FXSAVE/FXRSTOR.
    //    Set OSXMMEXCPT (bit 10) to enable unmasked SIMD FP exceptions (#XF).
    //    Set OSXSAVE (bit 18) if XSAVE is supported and to be used.
    let mut cr4: u32;
    asm!("mov {0}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
    if has_fxsr() { // FXSAVE/FXRSTOR are part of SSE generally
        cr4 |= 1 << 9;  // Set OSFXSR
    }
    if has_sse() { // If SSE is present, enable SIMD exceptions
        cr4 |= 1 << 10; // Set OSXMMEXCPT
    }
    if has_xsave() {
        cr4 |= 1 << 18; // Set OSXSAVE
        // Also, XCR0 needs to be configured to enable saving specific states (x87, SSE, AVX).
        // Default is usually 0x1 (x87) or 0x3 (x87+SSE) after boot.
        // To enable x87, SSE, AVX: asm!("mov ecx, 0", "mov eax, 7", "mov edx, 0", "xsetbv", options(nostack, preserves_flags));
        // This is complex and requires privileges. Usually set once by OS.
        // For now, assume XCR0 is reasonably configured by firmware/bootloader or needs specific setup.
    }
    asm!("mov cr4, {0}", in(reg) cr4, options(nomem, nostack, preserves_flags));

    // 3. Initialize FPU state (FINIT/FNINIT)
    // FNINIT is preferred as it doesn't wait for pending unmasked x87 exceptions.
    asm!("fninit", options(nomem, nostack, preserves_flags));

    // If using XSAVE, XCR0 should be set up. For example, to enable x87, SSE, and AVX:
    // if has_xsave() {
    //     unsafe {
    //         let xcr0_val: u64 = 0b111; // x87, SSE, AVX
    //         asm!(
    //             "mov ecx, 0", // XCR0 register index
    //             "mov eax, {val_low}",
    //             "mov edx, {val_high}",
    //             "xsetbv",
    //             val_low = in(reg) (xcr0_val & 0xFFFFFFFF) as u32,
    //             val_high = in(reg) (xcr0_val >> 32) as u32,
    //             options(nostack, preserves_flags)
    //         );
    //     }
    // }
}

/// Manages FPU state for a task, ensuring it's saved and restored correctly.
/// This is a simplified manager; a real one might handle lazy save/restore.
pub struct FpuManager {
    fpu_state: FpuState,
    active: bool, // Is this FPU state currently loaded into the FPU?
}

impl FpuManager {
    pub fn new() -> Self {
        Self {
            fpu_state: FpuState::new(), // Determines format based on CPU
            active: false,
        }
    }

    /// Call when a task is about to use FPU/SIMD.
    /// Restores its FPU state if not already active.
    pub unsafe fn begin_use(&mut self) {
        if !self.active {
            // TODO: Disable interrupts here if in a preemptive kernel
            self.fpu_state.restore();
            self.active = true;
            // TODO: Re-enable interrupts
        }
    }

    /// Call when a task is done with FPU/SIMD for now (e.g., before context switch out).
    /// Saves the FPU state if it was active.
    pub unsafe fn end_use(&mut self) {
        if self.active {
            // TODO: Disable interrupts
            self.fpu_state.save();
            self.active = false; // Mark as no longer loaded
            // TODO: Re-enable interrupts

            // Clear MMX state with EMMS if MMX was used and FSAVE wasn't the last save op.
            // This is tricky. If FSAVE was used, EMMS is implicit.
            // If FXSAVE/XSAVE, and MMX instructions were executed, EMMS is needed before
            // other FPU (x87) instructions or if switching to a task that might use x87.
            // For simplicity, if MMX is supported, and we just saved a potentially MMX-dirty state,
            // calling EMMS might be a safe bet if the next FPU user is unknown.
            // However, this is more about MMX instruction usage than state save/restore itself.
            // If the task itself is responsible for EMMS before calling FPU ops, then not needed here.
            // If this task was the last MMX user, EMMS.
            // if self.fpu_state.format != FpuStateFormat::FSAVE && has_mmx() {
            //    asm!("emms", options(nomem, nostack, preserves_flags));
            // }
        }
    }

    /// Returns the format of the saved FPU state.
    pub fn format(&self) -> FpuStateFormat {
        self.fpu_state.format
    }
}
