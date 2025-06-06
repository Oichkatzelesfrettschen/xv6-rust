#!/bin/bash
# Run xv6 in QEMU inside a tmux session to avoid terminal issues.
# Usage: ./run-tmux.sh [session-name]
# The script creates a detached tmux session and launches QEMU with a
# curses display so the output can be viewed by attaching to the session.

set -e

SESSION="${1:-xv6}"

# Build the image if it does not already exist.
[ -f xv6.img ] || make

# Start QEMU in a detached tmux session with a curses display.
tmux new-session -d -s "$SESSION" \
    "qemu-system-i386 -display curses -drive format=raw,file=xv6.img -serial mon:stdio"

echo "tmux session '$SESSION' started. Attach with: tmux attach -t $SESSION"
