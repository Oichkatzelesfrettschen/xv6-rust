/** Process management structures. */
use crate::file::{File, Inode};
use crate::mmu;
use crate::param;
use crate::types::Pde;

use core::ffi;

extern "C" {
    pub fn myproc() -> *const Proc;
    pub fn growproc(n: i32) -> i32;
    pub fn kill(pid: i32) -> i32;
    pub fn exit();
    pub fn fork() -> i32;
    pub fn sleep(chan: *const ffi::c_void, lk: *const ffi::c_void);
    pub fn wait() -> i32;
    pub fn procdump();
}

#[repr(C)]
/// Per-CPU state information.
pub struct Cpu {
    /// Local APIC ID for this CPU.
    apicid: u8,
    /// Scheduler context switch location.
    scheduler: *const Context,
    /// Task state segment for interrupts.
    ts: mmu::TaskState,
    /// Global descriptor table for this CPU.
    gdt: [mmu::SegDesc<u32>; param::NSEGS],
    /// Non-zero when CPU started.
    started: u32,
    /// Depth of pushcli nesting.
    ncli: i32,
    /// Interrupts enabled before pushcli.
    intena: i32,
    /// Currently running process on this CPU.
    proc: *const Proc,
}

#[repr(C)]
/// CPU context saved during kernel context switches.
#[derive(Debug, Copy, Clone)]
pub struct Context {
    /// Saved EDI register.
    edi: u32,
    /// Saved ESI register.
    esi: u32,
    /// Saved EBX register.
    ebx: u32,
    /// Saved EBP register.
    ebp: u32,
    /// Saved instruction pointer.
    eip: u32,
}

#[repr(C)]
/// Process state structure.
pub struct Proc {
    /// Size of process memory (bytes).
    pub sz: u32,
    /// Page table for this process.
    pub pgdir: *const Pde,
    /// Bottom of kernel stack for this process.
    pub kstack: *const u8,
    /// Process state (should be enum).
    pub procstate: u32,
    /// Process ID.
    pub pid: u32,
    /// Parent process.
    pub parent: *const Proc,
    /// CPU context for swtch().
    pub context: *const Context,
    /// If non-zero, sleeping on chan.
    pub chan: *const ffi::c_void,
    /// If non-zero, have been killed.
    pub killed: i32,
    /// Open files.
    pub ofile: [*const File; param::NOFILE],
    /// Current directory.
    pub cwd: *const Inode,
    /// Process name (debugging).
    pub name: [u8; 16],
}
