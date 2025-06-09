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

/// Minimal console writer used for output over the serial console.
#[derive(Default)]
pub struct Writer;

impl Writer {
    /// Construct a new [`Writer`].
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    /// Emit a single character to the UART, handling backspace specially.
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

    /// Low level write of a single byte.
    #[inline]
    fn write_byte(&self, byte: u8) {
        unsafe {
            uartputc(byte as i32);
        }
    }

    /// Output an entire string.
    pub fn write_string(&self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch as i32);
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
    /// Global console lock used by [`print`] macros.
    pub static ref CONSOLE: Mutex<Writer> = Mutex::new(Writer::new());
}

/// Print without a trailing newline.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::print(format_args!($($arg)*)));
}

/// Print with a trailing newline.
#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Low level printing routine used by [`print!`].
#[inline]
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
