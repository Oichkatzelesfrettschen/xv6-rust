# xv6-rust

This is a project to port the popular 32-bit learner's OS, xv6, over to the Rust programming language. I started this project as an undergraduate research project to gain more experience with operating systems and Rust.

One of the motivating academic factors behind this project (which has played a huge role in making this project possible for school credit) is assessing Rust's viability as a low level systems language.

[Click here](README) to see the original README that accompanied the xv6 revision 10 distribution.

[Click here](README-PDX) to see the original Portland State README that accompanied the course-specific xv6 distribution.

# Building and Running

Prerequisites:

1. A linux environment.

1. The QEMU simulator.

1. The `gcc` compiler suite with 32-bit support (`gcc-multilib g++-multilib`).

   These multilib packages provide the 32-bit C runtime used when cross
   compiling the kernel on a 64-bit host.

1. The Rust compiler.

1. `cargo-xbuild` (`cargo install cargo-xbuild`).

1. A nightly Rust toolchain installed (`rustup toolchain install nightly`).

1. The Rust source (`rustup component add rust-src`).

The build targets a 32-bit i386 system image that runs under QEMU.
Building on 64-bit hosts therefore requires the multilib C toolchain
or a cross compiler such as `i386-jos-elf-gcc`.

Building:
1. Run `make`.
2. Or use Meson with `meson setup build && ninja -C build`.

Running:

1. Run `make run`.
2. Run `./run-curses.sh` to start QEMU with a curses display.
   The script automatically launches a tmux session if none is active so
   you can detach and reattach later.
3. `./run-tmux.sh` can be invoked explicitly to create a named tmux session.

### Running in tmux

The `run-tmux.sh` helper creates a detached tmux session and launches
QEMU using a curses display. When `/dev/kvm` is available the script
enables hardware virtualization and uses two virtual CPUs for faster
booting. This is useful when the host terminal cannot display curses
directly or when you want to capture the boot output. Example usage:

```bash
./run-tmux.sh testsession
tmux attach -t testsession
```

Use `tmux capture-pane -pt testsession` to log the boot process.

### tmux configuration

The repository provides `xv6_tmux.conf` with a few quality-of-life options.
Run scripts will load this file automatically so custom settings do not
interfere with your personal `~/.tmux.conf`.

Debugging:

1. Run `make debug`; QEMU will expose a debugging port for GDB to attach to.

1. In another terminal session, run `rust-gdb`.
