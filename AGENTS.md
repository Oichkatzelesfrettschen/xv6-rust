Format all C code with clang-format and Rust code with rustfmt.
Provide Doxygen-style comments (/// or /** */) for every function.
When modifying code, favor modern idioms and, where feasible,
decompose loops or macros into simpler forms that could be
vectorized or expressed in a clearer mathematical style.

Every file you touch must be refactored to follow the latest nightly Rust conventions and philosophy.
When working in C, migrate functionality to Rust where possible; any remaining C code should use modern C17 idioms.

All code changes must pursue mathematical decomposition, unrolling, and refactoring into modern paradigms. Ensure every function includes thorough Doxygen comments, and integrate the documentation with Sphinx/Breathe for Read-the-Docs builds.
