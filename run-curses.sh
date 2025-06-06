#!/bin/bash
# Launch xv6 in QEMU using the curses display. The script builds the
# disk image if it is missing and then runs QEMU. Requires
# qemu-system-i386 with curses support.

set -e

# Build the image when absent.
[ -f xv6.img ] || make

qemu-system-i386 -display curses -drive format=raw,file=xv6.img -serial mon:stdio

