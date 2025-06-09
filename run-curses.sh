#!/bin/bash
# Launch xv6 in QEMU using the curses display. The script builds the
# disk image if it is missing and then runs QEMU. Requires
# qemu-system-i386 with curses support.

set -e

# If not already inside tmux, spawn a session so QEMU can be reattached.
if [ -z "$TMUX" ]; then
    exec "$(dirname "$0")/run-tmux.sh" "${1:-xv6}"
fi

# Build the image when absent.
[ -f xv6.img ] || make

qemu-system-i386 -display curses -drive format=raw,file=xv6.img -serial mon:stdio

