use crate::console::consoleintr;
use core::sync::atomic::AtomicUsize;
use x86::io::inb;

/// Basic PC keyboard driver mapping scan codes to characters.

// PC keyboard interface constants

const KBSTATP: u16 = 0x64; // kbd controller status port(I)
const KBS_DIB: u8 = 0x01; // kbd data in buffer
const KBDATAP: u16 = 0x60; // kbd data port(I)

const NO: u8 = 0;

// modifier bit for the shift key
const SHIFT: u8 = 1 << 0;
// modifier bit for the control key
const CTL: u8 = 1 << 1;
// modifier bit for the alt key
const ALT: u8 = 1 << 2;

// status bit for caps lock
const CAPSLOCK: u8 = 1 << 3;
// status bit for num lock
const NUMLOCK: u8 = 1 << 4;
// status bit for scroll lock
const SCROLLLOCK: u8 = 1 << 5;

// escape sequence indicator
const E0ESC: usize = 1 << 6;

// Special keycodes
const KEY_HOME: u8 = 0xE0;
const KEY_END: u8 = 0xE1;
const KEY_UP: u8 = 0xE2;
const KEY_DN: u8 = 0xE3;
const KEY_LF: u8 = 0xE4;
const KEY_RT: u8 = 0xE5;
const KEY_PGUP: u8 = 0xE6;
const KEY_PGDN: u8 = 0xE7;
const KEY_INS: u8 = 0xE8;
const KEY_DEL: u8 = 0xE9;

static SHIFT_VAR: AtomicUsize = AtomicUsize::new(0);
static CHARCODE: [&'static [u8]; 4] = [&NORMALMAP, &SHIFTMAP, &CTLMAP, &CTLMAP];

// translate a control character into the equivalent ASCII code
macro_rules! C {
    ($c:expr) => {
        $c as u8 - '@' as u8
    };
}

#[rustfmt::skip]
static NORMALMAP: [u8; 256] = [
    NO, 0x1B, b'1', b'2', b'3', b'4', b'5', b'6', // 0x00
    b'7', b'8', b'9', b'0', b'-', b'=', 0x08, b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u',
    b'i', // 0x10
    b'o', b'p', b'[', b']', b'\n', NO, b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l',
    b';', // 0x20
    b'\'', b'`', NO, b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', NO,
    b'*', // 0x30
    NO, ' ' as u8, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, b'7', // 0x40
    b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', b'2', b'3', b'0', b'.', NO, NO, NO,
    NO, // 0x50
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x60
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x70
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x80
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, KEY_HOME, // 0x90
    NO, NO, NO, NO, b'\n', NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xA0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, b'/', NO, NO, // 0xB0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xC0
    KEY_UP, KEY_PGUP, NO, KEY_LF, NO, NO, NO, KEY_END, KEY_DN, KEY_PGDN, KEY_INS, KEY_DEL, NO,
    KEY_RT, NO, NO, // 0xD0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xE0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xF0
    NO, NO, NO, NO, NO, NO, NO, NO,
];

#[rustfmt::skip]
static SHIFTMAP: [u8; 256] = [
    NO, 033, b'!', b'@', b'#', b'$', b'%', b'^', // 0x00
    b'&', b'*', b'(', b')', b'_', b'+', 0x08, b'\t', b'Q', b'W', b'E', b'R', b'T', b'Y', b'U',
    b'I', // 0x10
    b'O', b'P', b'{', b'}', b'\n', NO, b'A', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L',
    b':', // 0x20
    b'"', b'~', NO, b'|', b'Z', b'X', b'C', b'V', b'B', b'N', b'M', b'<', b'>', b'?', NO,
    b'*', // 0x30
    NO, ' ' as u8, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, b'7', // 0x40
    b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', b'2', b'3', b'0', b'.', NO, NO, NO,
    NO, // 0x50
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x60
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x70
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x80
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, KEY_HOME, // 0x90
    NO, NO, NO, NO, b'\n', NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xA0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, b'/', NO, NO, // 0xB0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xC0
    KEY_UP, KEY_PGUP, NO, KEY_LF, NO, NO, NO, KEY_END, KEY_DN, KEY_PGDN, KEY_INS, KEY_DEL, NO,
    KEY_RT, NO, NO, // 0xD0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xE0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xF0
    NO, NO, NO, NO, NO, NO, NO, NO,
];

#[rustfmt::skip]
static CTLMAP: [u8; 256] = [
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO, // 0x00
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    C!('Q'),
    C!('W'),
    C!('E'),
    C!('R'),
    C!('T'),
    C!('Y'),
    C!('U'),
    C!('I'), // 0x10
    C!('O'),
    C!('P'),
    NO,
    NO,
    b'\r',
    NO,
    C!('A'),
    C!('S'),
    C!('D'),
    C!('F'),
    C!('G'),
    C!('H'),
    C!('J'),
    C!('K'),
    C!('L'),
    NO, // 0x20
    NO,
    NO,
    NO,
    C!('\\'),
    C!('Z'),
    C!('X'),
    C!('C'),
    C!('V'),
    C!('B'),
    C!('N'),
    C!('M'),
    NO,
    NO,
    239,
    NO,
    NO, // 0x30
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO, // 0x40
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO, // 0x50
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO, // 0x60
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO, // 0x70
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO, // 0x80
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    KEY_HOME, // 0x90
    NO,
    NO,
    NO,
    NO,
    b'\r',
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO, // 0xA0
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    239,
    NO,
    NO, // 0xB0
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO, // 0xC0
    KEY_UP,
    KEY_PGUP,
    NO,
    KEY_LF,
    NO,
    NO,
    NO,
    KEY_END,
    KEY_DN,
    KEY_PGDN,
    KEY_INS,
    KEY_DEL,
    NO,
    KEY_RT,
    NO,
    NO, // 0xD0
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO, // 0xE0
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO, // 0xF0
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
    NO,
];

#[rustfmt::skip]
static SHIFTCODE: [u8; 256] = [
    NO, NO, NO, NO, NO, NO, NO, NO, // 0x00
    NO, NO, NO, NO, NO, CTL, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x10
    NO, NO, NO, NO, NO, CTL, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x20
    NO, NO, SHIFT, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, SHIFT, NO, // 0x30
    ALT, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x40
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x50
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x60
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x70
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x80
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x90
    NO, NO, NO, NO, NO, CTL, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xA0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xB0
    ALT, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xC0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xD0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xE0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xF0
    NO, NO, NO, NO, NO, NO, NO, NO,
];

#[rustfmt::skip]
static TOGGLECODE: [u8; 256] = [
    NO, NO, NO, NO, NO, NO, NO, NO, // 0x00
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x10
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x20
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x30
    NO, NO, CAPSLOCK, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NUMLOCK, SCROLLLOCK, NO, // 0x40
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x50
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x60
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x70
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x80
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x90
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xA0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xB0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xC0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xD0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xE0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xF0
    NO, NO, NO, NO, NO, NO, NO, NO,
];

#[no_mangle]
/// Keyboard interrupt handler invoked from ASM stub.
pub extern "C" fn kbdintr() {
    unsafe {
        consoleintr(kbdgetc);
    }
}

/// Read a key from the keyboard controller.
pub extern "C" fn kbdgetc() -> i32 {
    use core::sync::atomic::Ordering;
    let st = unsafe { inb(KBSTATP as u16) };
    if st & KBS_DIB == 0 {
        return -1;
    }
    let mut data = unsafe { inb(KBDATAP as u16) };
    if data == 0xE0 {
        SHIFT_VAR.fetch_or(E0ESC, Ordering::SeqCst);
        return 0;
    } else if data & 0x80 != 0 {
        // Key released
        data = if SHIFT_VAR.load(Ordering::SeqCst) & E0ESC != 0 {
            data
        } else {
            data & 0x7F
        };
        SHIFT_VAR.fetch_and(
            !(SHIFTCODE[data as usize] as usize | E0ESC),
            Ordering::SeqCst,
        );
        return 0;
    } else if SHIFT_VAR.load(Ordering::SeqCst) & E0ESC != 0 {
        // Last character was an E0 escape; or with 0x80
        data |= 0x80;
        SHIFT_VAR.fetch_and(!E0ESC, Ordering::SeqCst);
    }
    let mut state = SHIFT_VAR.load(Ordering::SeqCst);
    state |= SHIFTCODE[data as usize] as usize;
    state ^= TOGGLECODE[data as usize] as usize;
    SHIFT_VAR.store(state, Ordering::SeqCst);

    let mut c = CHARCODE[state & (CTL | SHIFT) as usize][data as usize];
    if state & CAPSLOCK as usize != 0 {
        if b'a' <= c && c <= b'z' {
            c += 224 // 'A' - 'a'
        } else if b'A' <= c && c <= b'Z' {
            c += b'a' - b'A';
        }
    }
    c as i32
}
