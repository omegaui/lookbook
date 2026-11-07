#!/bin/sh
# Saves a screenshot of only the Linux & You window. Needs Hyprland (hyprctl) and grim.
# It reads the window's position and captures that area; it never moves the mouse.
#
#   .agentic-coding/scripts/screenshot-window.sh /tmp/explore.png
set -eu

OUTPUT=${1:-/tmp/linux-and-you.png}

GEOMETRY=$(hyprctl clients -j | python3 -I -c '
import json, sys
windows = [w for w in json.load(sys.stdin) if "LinuxAndYou" in w["class"]]
if windows:
    w = windows[0]
    print("%d,%d %dx%d" % (w["at"][0], w["at"][1], w["size"][0], w["size"][1]))
')

if [ -z "$GEOMETRY" ]; then
    echo "The Linux & You window is not open." >&2
    exit 1
fi

grim -g "$GEOMETRY" "$OUTPUT"
echo "$OUTPUT"
