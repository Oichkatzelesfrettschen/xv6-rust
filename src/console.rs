//! \file console.rs
//! \brief Simple UART-backed console utilities.

use crate::spinlock;
use crate::uart::uartputc;
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;

extern "C" {
    /// \brief Pointer to the global spinlock for console.
    static conslk: *const spinlock::Spinlock;

    /// \brief Register the console interrupt handler.
    ///
    /// \param f Function pointer to call on console interrupt.
    pub fn consoleintr(f: unsafe extern "C" fn() -> i32);
}

/// \brief Special code indicating a backspace in raw input.
const BACKSPACE: i32 = 0x100;
/// \brief ASCII backspace byte for output.
const BACKSCHAR: u8 = b'\x08';

/// \brief Minimal console writer used for output over the serial console.
#[derive(Default)]
pub struct Writer;

impl Writer {
    /// \brief Construct a new [`Writer`].
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    /// \brief Emit a single character to the UART, handling backspace specially.
    ///
    /// \param ch Character code or `BACKSPACE`.
    #[inline]
    pub fn write_char(&self, ch: i32) {
        if ch == BACKSPACE {
            self.write_byte(BACKSCHAR);
            self.write_byte(b' ');
            self.write_byte(BACKSCHAR);
        } else {
            self.write_byte(ch as u8);
        }
    }

    /// \brief Low-level write of a single byte to UART.
    ///
    /// \param byte Byte value to send.
    #[inline]
    fn write_byte(&self, byte: u8) {
        unsafe { uartputc(byte as i32); }
    }

    /// \brief Write a string slice to the console.
    ///
    /// \param s String slice to output.
    pub fn write_string(&self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch as i32);
        }
    }
}

/// \brief Implement the `fmt::Write` trait for console output.
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    /// \brief Global console lock used by `print!` and `println!` macros.
    pub static ref CONSOLE: Mutex<Writer> = Mutex::new(Writer::new());
}

/// \brief Print formatted text without a trailing newline.
///
/// Usage: `print!("Value = {}", x);`
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::print(format_args!($($arg)*))
    };
}

/// \brief Print formatted text followed by a newline.
///
/// Usage: `println!("Hello, {}!", name);`
#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// \brief Low-level printing routine used by `print!` macro.
///
/// Acquires the console spinlock, writes the formatted arguments, then
/// releases the lock.
///
/// \param args Pre-formatted arguments.
#[inline]
pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe { spinlock::acquire(conslk); }
    CONSOLE.lock().write_fmt(args).unwrap();
    unsafe { spinlock::release(conslk); }
}
