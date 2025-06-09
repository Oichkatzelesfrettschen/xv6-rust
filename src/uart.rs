//! \file uart.rs
//! \brief Minimal driver for the Intel 8250 serial port (UART).
//!
//! Provides early-boot console I/O and interrupt handling over COM1.
//!  
//! Reference: https://wiki.osdev.org/UART

use crate::console::consoleintr;
use crate::ioapic::ioapicenable;
use crate::lapic::microdelay;
use crate::traps::IRQ_COM1;
use x86::io::{inb, outb};

/// \brief I/O port base address for the first serial port (COM1).
const COM1: u16 = 0x3F8;

/// \brief Flag indicating whether the UART has been detected and initialized.
static mut UART_PRESENT: bool = false;

/// \brief Initialize the 8250 UART for 9600 baud, 8 data bits, 1 stop bit, no parity,
/// and enable receive interrupts.
///  
/// - Disables the FIFO.
/// - Unlocks divisor latch and sets divisor for 9600 baud.
/// - Configures 8N1 frame format.
/// - Enables RX interrupts.
/// - Verifies presence by checking Line Status Register.
/// - Clears any pending interrupts, registers IRQ with IO APIC.
/// - Sends a startup banner.
///  
/// \note Must be called early in boot, before interrupts are enabled.
#[no_mangle]
#[inline]
pub unsafe extern "C" fn uartinit() {
    // Turn off the FIFO
    outb(COM1 + 2, 0x00);

    // 9600 baud, 8 data bits, 1 stop bit, parity off
    outb(COM1 + 3, 0x80);       // Unlock divisor latch
    outb(COM1 + 0, 12);         // Divisor low byte (9600)
    outb(COM1 + 1, 0x00);       // Divisor high byte
    outb(COM1 + 3, 0x03);       // Lock divisor, 8 data bits, no parity
    outb(COM1 + 4, 0x00);       // Disable FIFO / special modes
    outb(COM1 + 1, 0x01);       // Enable receive data available interrupt

    // Detect absence: LSR == 0xFF => no UART
    if inb(COM1 + 5) == 0xFF {
        return;
    }
    UART_PRESENT = true;

    // Acknowledge existing interrupts and register COM1 IRQ
    let _ = inb(COM1 + 2);
    let _ = inb(COM1 + 0);
    ioapicenable(IRQ_COM1, 0);

    // Announce via UART
    for &b in b"xv6...\n" {
        uartputc(b as i32);
    }
}

/// \brief Output a single character via the UART.
///  
/// Blocks (with a brief spin + microdelay) until the transmitter FIFO has room,
/// then writes the byte. No-op if UART not present.
///  
/// \param c Character code (0–255).
#[no_mangle]
#[inline]
pub unsafe extern "C" fn uartputc(c: i32) {
    if !UART_PRESENT {
        return;
    }
    // Wait for Transmitter Holding Register Empty (bit 5 of LSR)
    for _ in 0..128 {
        if (inb(COM1 + 5) & 0x20) != 0 {
            break;
        }
        microdelay(10);
    }
    outb(COM1 + 0, c as u8);
}

/// \brief Read a character from the UART if available.
///  
/// Returns the received byte (0–255), or –1 if no data or UART absent.
///  
/// \returns Received byte or –1.
#[no_mangle]
#[inline]
pub unsafe extern "C" fn uartgetc() -> i32 {
    if !UART_PRESENT {
        return -1;
    }
    // Data Ready if bit 0 of LSR set
    if (inb(COM1 + 5) & 0x01) == 0 {
        return -1;
    }
    inb(COM1 + 0) as i32
}

/// \brief UART interrupt handler: feed incoming bytes into console layer.
///  
/// Invoked on COM1 IRQ; reads all available bytes and passes them to `consoleintr`.
#[no_mangle]
#[inline]
pub unsafe extern "C" fn uartintr() {
    consoleintr(uartgetc);
}
