# xv6-rust

This is a project to port the popular 32-bit learner's OS, xv6, over to the Rust programming language. I started this project as an undergraduate research project to gain more experience with operating systems and Rust.

One of the motivating academic factors behind this project (which has played a huge role in making this project possible for school credit) is assessing Rust's viability as a low level systems language.

[Click here](README) to see the original README that accompanied the xv6 revision 10 distribution.

[Click here](README-PDX) to see the original Portland State README that accompanied the course-specific xv6 distribution.

# Building and Running

Prerequisites:

1. A linux environment.

1. The QEMU simulator.

1. The `gcc` compiler suite.

1. The Rust compiler.

1. `cargo-xbuild` (`cargo install cargo-xbuild`).

1. A nightly override for the cloned repository (`rustup override set nightly`).

1. The Rust source (`rustup component add rust-src`).

## Running setup.sh

Run `./setup.sh` before building to install all dependencies automatically. The script installs QEMU, Sphinx, Doxygen, CLOC, and the Rust toolchains (including rustfmt) along with other utilities, preparing the environment for compilation and documentation generation.

Building:
1. Run `make`.
2. Or use Meson with `meson setup build && ninja -C build`.

Running:

1. Run `make run`.
2. Or run `./run-curses.sh` to start QEMU with a curses display.
3. Containers that can't display curses directly can use `./run-tmux.sh`.

### Running in tmux

The `run-tmux.sh` helper creates a detached tmux session and launches
QEMU using a curses display. This is useful when the host terminal
cannot display curses directly or when you want to capture the boot
output. Example usage:

```bash
./run-tmux.sh testsession
tmux attach -t testsession
```

Use `tmux capture-pane -pt testsession` to log the boot process.

Debugging:

1. Run `make debug`; QEMU will expose a debugging port for GDB to attach to.

1. In another terminal session, run `rust-gdb`.

## Contribution Guidelines

All code changes must pursue mathematical decomposition, unrolling loops where it clarifies the intent, and refactor code into modern paradigms. Every function must include thorough Doxygen comments, and documentation should integrate with Sphinx and Breathe to support Read-the-Docs builds.
