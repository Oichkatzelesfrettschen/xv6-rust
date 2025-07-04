// Console input and output.
// Input is from the keyboard or serial port.
// Output is written to the screen and serial port.
// clang-format off
#include "types.h"
#include "defs.h"
#include "param.h"
#include "memlayout.h"
#include "mmu.h"
#include "proc.h"
#include "spinlock.h"
#include "sleeplock.h"
#include "fs.h"
#include "file.h"
#include "traps.h"
#include "x86.h"
// clang-format on

static void consputc(int);

static int panicked = 0;

static struct {
  struct spinlock lock;
  int locking;
} cons;

// Enables `console.rs` to use xv6's locking primitives
struct spinlock *conslk = &cons.lock;

// Forward declarations for compiler built-ins used in this file
extern void *memmove(void *dst, const void *src, uint n);
extern void *memset(void *dst, int c, uint n);

/**
 * @brief Print an integer value to the console.
 *
 * @param xx   Integer to print.
 * @param base Numerical base (e.g. 10 for decimal).
 * @param sign Non-zero if the value is signed.
 */
static void printint(int xx, int base, int sign) {
  static char digits[] = "0123456789abcdef";
  char buf[16];
  int i;
  uint x;

  if (sign && (sign = xx < 0))
    x = -xx;
  else
    x = xx;

  i = 0;
  do {
    buf[i++] = digits[x % base];
  } while ((x /= base) != 0);

  if (sign)
    buf[i++] = '-';

  while (--i >= 0)
    consputc(buf[i]);
}
// PAGEBREAK: 50

/**
 * @brief Formatted console output.
 *
 * Supported format specifiers: %d, %x, %p, %s.
 *
 * @param fmt Format string.
 * @param ... Additional arguments matching the format string.
 */
void cprintf(char *fmt, ...) {
  int i, c, locking;
  uint *argp;
  char *s;

  locking = cons.locking;
  if (locking)
    acquire(&cons.lock);

  if (fmt == 0)
    panic("null fmt");

  argp = (uint *)(void *)(&fmt + 1);
  for (i = 0; (c = fmt[i] & 0xff) != 0; i++) {
    if (c != '%') {
      consputc(c);
      continue;
    }
    c = fmt[++i] & 0xff;
    if (c == 0)
      break;
    switch (c) {
    case 'd':
      printint(*argp++, 10, 1);
      break;
    case 'x':
    case 'p':
      printint(*argp++, 16, 0);
      break;
    case 's':
      if ((s = (char *)*argp++) == 0)
        s = "(null)";
      for (; *s; s++)
        consputc(*s);
      break;
    case '%':
      consputc('%');
      break;
    default:
      // Print unknown % sequence to draw attention.
      consputc('%');
      consputc(c);
      break;
    }
  }

  if (locking)
    release(&cons.lock);
}

/**
 * @brief Halt the system and print a panic message.
 *
 * This function does not return.
 *
 * @param s Panic message string.
 */
void panic(char *s) {
  int i;
  uint pcs[10];

  cli();
  cons.locking = 0;
  // use lapiccpunum so that we can call panic from mycpu()
  cprintf("lapicid %d: panic: ", lapicid());
  cprintf(s);
  cprintf("\n");
  getcallerpcs(&s, pcs);
  for (i = 0; i < 10; i++)
    cprintf(" %p", pcs[i]);
  panicked = 1; // freeze other CPU
  for (;;)
    ;
}

// PAGEBREAK: 50
#define BACKSPACE 0x100
#define CRTPORT 0x3d4
static ushort *crt = (ushort *)P2V(0xb8000); // CGA memory

/**
 * @brief Output a character to the CGA console.
 *
 * Handles scrolling and cursor movement.
 *
 * @param c Character to display.
 */
static void cgaputc(int c) {
  int pos;

  // Cursor position: col + 80*row.
  outb(CRTPORT, 14);
  pos = inb(CRTPORT + 1) << 8;
  outb(CRTPORT, 15);
  pos |= inb(CRTPORT + 1);

  if (c == '\n')
    pos += 80 - pos % 80;
  else if (c == BACKSPACE) {
    if (pos > 0)
      --pos;
  } else
    crt[pos++] = (c & 0xff) | 0x0700; // black on white

  if (pos < 0 || pos > 25 * 80)
    panic("pos under/overflow");

  if ((pos / 80) >= 24) { // Scroll up.
    memmove(crt, crt + 80, sizeof(crt[0]) * 23 * 80);
    pos -= 80;
    memset(crt + pos, 0, sizeof(crt[0]) * (24 * 80 - pos));
  }

  outb(CRTPORT, 14);
  outb(CRTPORT + 1, pos >> 8);
  outb(CRTPORT, 15);
  outb(CRTPORT + 1, pos);
  crt[pos] = ' ' | 0x0700;
}

/**
 * @brief Output a character to both UART and screen.
 *
 * Handles backspace mirroring to UART and halts if the system panicked.
 *
 * @param c Character to output.
 */
void consputc(int c) {
  if (panicked) {
    cli();
    for (;;)
      ;
  }

  if (c == BACKSPACE) {
    uartputc('\b');
    uartputc(' ');
    uartputc('\b');
  } else
    uartputc(c);
  cgaputc(c);
}

#define INPUT_BUF 128
struct {
  char buf[INPUT_BUF];
  uint r; // Read index
  uint w; // Write index
  uint e; // Edit index
} input;

#define C(x) ((x) - '@') // Control-x

/**
 * @brief Console interrupt handler.
 *
 * Reads characters using the provided getter and processes line editing
 * commands. Wakes up processes waiting for console input.
 *
 * @param getc Input function returning a character or -1.
 */
void consoleintr(int (*getc)(void)) {
  int c, doprocdump = 0;

  acquire(&cons.lock);
  while ((c = getc()) >= 0) {
    switch (c) {
    case C('P'): // Process listing.
      // procdump() locks cons.lock indirectly; invoke later
      doprocdump = 1;
      break;
    case C('U'): // Kill line.
      while (input.e != input.w &&
             input.buf[(input.e - 1) % INPUT_BUF] != '\n') {
        input.e--;
        consputc(BACKSPACE);
      }
      break;
    case C('H'):
    case '\x7f': // Backspace
      if (input.e != input.w) {
        input.e--;
        consputc(BACKSPACE);
      }
      break;
    default:
      if (c != 0 && input.e - input.r < INPUT_BUF) {
        c = (c == '\r') ? '\n' : c;
        input.buf[input.e++ % INPUT_BUF] = c;
        consputc(c);
        if (c == '\n' || c == C('D') || input.e == input.r + INPUT_BUF) {
          input.w = input.e;
          wakeup(&input.r);
        }
      }
      break;
    }
  }
  release(&cons.lock);
  if (doprocdump) {
    procdump(); // now call procdump() wo. cons.lock held
  }
}

/**
 * @brief Read characters from the console input buffer.
 *
 * Blocks until data is available or the process is killed.
 *
 * @param ip  Inode for the device (unlocked on entry).
 * @param dst Destination buffer.
 * @param n   Maximum number of bytes to read.
 * @return Number of bytes read or -1 on kill.
 */
int consoleread(struct inode *ip, char *dst, int n) {
  uint target;
  int c;

  iunlock(ip);
  target = n;
  acquire(&cons.lock);
  while (n > 0) {
    while (input.r == input.w) {
      if (myproc()->killed) {
        release(&cons.lock);
        ilock(ip);
        return -1;
      }
      sleep(&input.r, &cons.lock);
    }
    c = input.buf[input.r++ % INPUT_BUF];
    if (c == C('D')) { // EOF
      if (n < target) {
        // Save ^D for next time, to make sure
        // caller gets a 0-byte result.
        input.r--;
      }
      break;
    }
    *dst++ = c;
    --n;
    if (c == '\n')
      break;
  }
  release(&cons.lock);
  ilock(ip);

  return target - n;
}

/**
 * @brief Write a buffer to the console.
 *
 * @param ip  Inode for the device (unlocked on entry).
 * @param buf Buffer to write.
 * @param n   Number of bytes.
 * @return Number of bytes written.
 */
int consolewrite(struct inode *ip, char *buf, int n) {
  int i;

  iunlock(ip);
  acquire(&cons.lock);
  for (i = 0; i < n; i++)
    consputc(buf[i] & 0xff);
  release(&cons.lock);
  ilock(ip);

  return n;
}

/**
 * @brief Initialize the console subsystem.
 *
 * Sets up locks and enables keyboard interrupts.
 */
void consoleinit(void) {
  initlock(&cons.lock, "console");

  devsw[CONSOLE].write = consolewrite;
  devsw[CONSOLE].read = consoleread;
  cons.locking = 1;

  ioapicenable(IRQ_KBD, 0);
}
