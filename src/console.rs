//! \file console.rs
//! \brief Simple UART-backed console utilities.

use crate::spinlock;
use crate::uart::uartputc;

use lazy_static::lazy_static;
use spin::Mutex;

use core::fmt;

extern "C" {
    static conslk: *const spinlock::Spinlock;
    pub fn consoleintr(f: unsafe extern "C" fn() -> i32);
}

/// Constant representing the backspace key in raw mode.
const BACKSPACE: i32 = 0x100;
const BACKSCHAR: u8 = b'\x08';

/// \brief Minimal console writer used by `print!` macros.
pub struct Writer;

impl Writer {
    /// \brief Create a new console writer.
    pub const fn new() -> Self {
        Writer {}
    }

    /// \brief Output a single character to the UART.
    pub fn write_char(&self, ch: i32) {
        if ch == BACKSPACE {
            self.write_byte(BACKSCHAR);
            self.write_byte(b' ');
            self.write_byte(BACKSCHAR);
        } else {
            self.write_byte(ch as u8);
        }
    }

    /// \brief Send a raw byte to the UART.
    fn write_byte(&self, byte: u8) {
        unsafe {
            uartputc(byte as i32);
        }
    }

    /// \brief Write a string to the console.
    pub fn write_string(&self, s: &str) {
        for byte in s.bytes() {
            self.write_char(byte as i32);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Writer> = Mutex::new(Writer::new());
}

/// \brief Write formatted text without a trailing newline.
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::print(format_args!($($arg)*)));
}

/// \brief Write formatted text followed by a newline.
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// \brief Print formatted arguments to the console.
pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        spinlock::acquire(conslk);
    }
    CONSOLE.lock().write_fmt(args).unwrap();
    unsafe {
        spinlock::release(conslk);
    }
}
