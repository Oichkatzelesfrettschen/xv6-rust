extern "C" {
    /// Enable an interrupt line in the I/O APIC.
    ///
    /// # Parameters
    /// - `irq`: Interrupt request number to enable.
    /// - `cpunum`: Target CPU's APIC ID that should service the interrupt.
    pub fn ioapicenable(irq: i32, cpunum: i32);
}
