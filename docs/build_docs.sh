#!/bin/bash
# -----------------------------------------------------------------------------
# Build Doxygen and Sphinx documentation, aborting if any warnings are found.
# -----------------------------------------------------------------------------

set -euo pipefail

# Capture Doxygen output for post-processing.
doxygen_log=$(mktemp)

# Generate API documentation from source using Doxygen.
doxygen docs/Doxyfile 2>&1 | tee "$doxygen_log"

# Fail the build when Doxygen emits warnings.
if grep -Ei "warning:" "$doxygen_log" >/dev/null; then
	echo "Doxygen reported warnings." >&2
	rm -f "$doxygen_log"
	exit 1
fi
rm -f "$doxygen_log"

# Build Sphinx documentation with warnings treated as errors.
make -C docs SPHINXOPTS=-W html
