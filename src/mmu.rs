use bitfield::bitfield;
use core::ffi;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
/// Task state segment for hardware task switching.
pub struct TaskState {
    /// Old link field.
    link: u32,
    /// Stack pointer for ring 0.
    esp0: u32,
    /// Stack segment for ring 0.
    ss0: u16,
    /// Reserved padding.
    padding1: u16,
    /// Stack pointer for ring 1.
    esp1: *const u32,
    /// Stack segment for ring 1.
    ss1: u16,
    /// Reserved padding.
    padding2: u16,
    /// Stack pointer for ring 2.
    esp2: *const u32,
    /// Stack segment for ring 2.
    ss2: u16,
    /// Reserved padding.
    padding3: u16,
    /// CR3 value.
    cr3: *const ffi::c_void,
    /// Instruction pointer.
    eip: *const u32,
    /// EFLAGS register.
    eflags: u32,
    /// Saved EAX register.
    eax: u32,
    /// Saved ECX register.
    ecx: u32,
    /// Saved EDX register.
    edx: u32,
    /// Saved EBX register.
    ebx: u32,
    /// Saved stack pointer.
    esp: *const u32,
    /// Saved frame pointer.
    ebp: *const u32,
    /// Saved source index.
    esi: u32,
    /// Saved destination index.
    edi: u32,
    /// Segment register ES.
    es: u16,
    /// Reserved padding.
    padding4: u16,
    /// Code segment selector.
    cs: u16,
    /// Reserved padding.
    padding5: u16,
    /// Stack segment selector.
    ss: u16,
    /// Reserved padding.
    padding6: u16,
    /// Data segment selector.
    ds: u16,
    /// Reserved padding.
    padding7: u16,
    /// FS segment selector.
    fs: u16,
    /// Reserved padding.
    padding8: u16,
    /// GS segment selector.
    gs: u16,
    /// Reserved padding.
    padding9: u16,
    /// Local descriptor table selector.
    ldt: u16,
    /// Reserved padding.
    padding10: u16,
    /// Trap flag.
    t: u16,
    /// I/O map base address.
    iomb: u16,
}

bitfield! {
    #[repr(C)]
    /// x86 segment descriptor.
    pub struct SegDesc(MSB0 [u8]);
    u32;
    get_lim_15_0, _: 15, 0;
    get_base_15_0, _: 31, 16;
    base_23_16, _: 39, 32;
    segtype, _: 43, 40;
    s, _: 44;
    dpl, _: 46, 45;
    p, _: 47;
    lim_19_16, _: 51, 48;
    avl, _: 52;
    rsv1, _: 53;
    db, _: 54;
    g, _: 55;
    base_31_24, _: 63, 56;
}
