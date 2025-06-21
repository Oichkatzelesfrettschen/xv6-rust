//! \file arch.rs
//! \brief Architecture-specific structures and constants.

use bytemuck::{Pod, Zeroable};

#[repr(C)]
/// \brief CPU state saved during a trap.
/// Saved registers for trap handling.
#[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
pub struct Trapframe {
    edi: u32,
    esi: u32,
    ebp: u32,
    oesp: u32,
    ebx: u32,
    edx: u32,
    ecx: u32,
    eax: u32,
    gs: u16,
    padding1: u16,
    fs: u16,
    padding2: u16,
    es: u16,
    padding3: u16,
    ds: u16,
    padding4: u16,
    trapno: u32,
    err: u32,
    eip: u32,
    cs: u16,
    padding5: u16,
    eflags: u32,
    esp: u32,
    ss: u16,
    padding6: u16,
}
