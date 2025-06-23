# xv6-rust

This is a project to port the popular 32-bit learner's OS, xv6, over to the Rust programming language. I started this project as an undergraduate research project to gain more experience with operating systems and Rust.

One of the motivating academic factors behind this project (which has played a huge role in making this project possible for school credit) is assessing Rust's viability as a low level systems language.

[Click here](README) to see the original README that accompanied the xv6 revision 10 distribution.

[Click here](README-PDX) to see the original Portland State README that accompanied the course-specific xv6 distribution.

# Building and Running

Prerequisites:

1. A Linux environment.
2. The QEMU simulator.
3. The `gcc` compiler suite.
4. A nightly Rust toolchain (e.g., run `rustup default nightly` or `rustup override set nightly` in the project directory).
5. The Rust source component (`rustup component add rust-src`).
6. `cargo-xbuild` (`cargo install cargo-xbuild`) - **Note: Only required if building with Meson. The `make` build process uses `-Z build-std` flags directly.**

## Running setup.sh

Run `./setup.sh` before building to install all dependencies automatically. The script installs QEMU, Sphinx, Doxygen, CLOC, and the Rust toolchains (including rustfmt) along with other utilities, preparing the environment for compilation and documentation generation.

Building:
1. Run `make`. This uses `cargo build` with specific flags (`-Z build-std=...`) to compile the Rust code for the custom target, requiring a nightly toolchain and the `rust-src` component.
2. Or use Meson with `meson setup build && ninja -C build`. This method uses `cargo xbuild`.
3. The default configuration sets `CFLAGS` to `-O3 -march=native -pipe` and applies
   corresponding `RUSTFLAGS` (`-C target-cpu=native -C opt-level=3 -C link-arg=-Wl,--gc-sections`)
   via `.cargo/config.toml` for optimized, size-trimmed binaries.

(For developers using direct `cargo` commands, e.g., in IDEs, the project includes a `.cargo/config.toml` file that configures the build for the custom target automatically when using a nightly toolchain.)

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
