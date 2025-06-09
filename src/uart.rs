//! \file uart.rs
//! \brief Minimal driver for the Intel 8250 serial port (UART).
//!
//! This module provides early boot console output and input handling
//! used throughout the kernel.
// https://wiki.osdev.org/UART

use crate::console::consoleintr;
use crate::ioapic::ioapicenable;
use crate::lapic::microdelay;
use crate::traps::IRQ_COM1;
use x86::io::{inb, outb};

/// UART driver used for early boot and debugging output.

const COM1: u16 = 0x3f8;

static mut UART_PRESENT: bool = false;

#[no_mangle]
pub unsafe extern "C" fn uartinit() {
    // Turn off the FIFO
    outb(COM1 + 2, 0);

    // 9600 baud, 8 data bits, 1 stop bit, parity off.
    outb(COM1 + 3, 0x80); // Unlock divisor
    outb(COM1 + 0, 12);
    outb(COM1 + 1, 0);
    outb(COM1 + 3, 0x03); // Lock divisor, 8 data bits.
    outb(COM1 + 4, 0);
    outb(COM1 + 1, 0x01); // Enable receive interrupts.

    // If status is 0xFF, no serial port.
    if inb(COM1 + 5) == 0xFF {
        return;
    }

    UART_PRESENT = true;

    // Acknowledge pre-existing interrupt conditions;
    // enable interrupts.
    inb(COM1 + 2);
    inb(COM1 + 0);
    ioapicenable(IRQ_COM1, 0);

    // Announce that we're here.
    for ch in b"xv6...\n" {
        uartputc(*ch as i32);
    }
}

#[no_mangle]
/// \brief Output a single character via the UART.
///
/// Blocks until the transmit FIFO has space and then writes the byte.
pub unsafe extern "C" fn uartputc(c: i32) {
    if !UART_PRESENT {
        return;
    }

    for _i in 0..128 {
        if (inb(COM1 + 5) & 0x20) != 0 {
            break;
        }

        microdelay(10);
    }

    outb(COM1 + 0, c as u8);
}

#[no_mangle]
/// \brief Read a character from the UART if available.
pub unsafe extern "C" fn uartgetc() -> i32 {
    if !UART_PRESENT {
        return -1;
    }
    // Return -1 if the receive buffer is empty
    if (inb(COM1 + 5) & 0x01) == 0 {
        return -1;
    }

    inb(COM1 + 0) as i32
}

#[no_mangle]
/// \brief UART interrupt handler that feeds received bytes to the console layer.
pub unsafe extern "C" fn uartintr() {
    consoleintr(uartgetc);
}
