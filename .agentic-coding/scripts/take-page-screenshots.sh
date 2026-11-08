#!/bin/sh
# Retakes the screenshots in data/screenshots/ (shown on the app page) from
# the installed release app, driven by its scripting actions. Needs Hyprland
# and grim; it never moves the mouse. Check every picture afterwards: if the
# window is not on screen yet, the capture shows whatever is there instead.
#
#   .agentic-coding/scripts/take-page-screenshots.sh
set -eu

PROJECT=$(cd "$(dirname "$0")/../.." && pwd)
APP=io.github.omegaui.Syspedia
OUT="$PROJECT/data/screenshots"
SHOT="$PROJECT/.agentic-coding/scripts/screenshot-window.sh"

start_on() {
    flatpak kill "$APP" 2>/dev/null || true
    sleep 1
    flatpak run --command=gsettings "$APP" set "$APP" last-page "$1"
    flatpak run --env=SYSPEDIA_SCRIPTING=1 "$APP" >/dev/null 2>&1 &
    sleep "${2:-8}"
}

act() {
    flatpak run --command=gapplication "$APP" action "$APP" "$@"
}

take() {
    sleep "$2"
    "$SHOT" "$OUT/$1.png" >/dev/null
    echo "took $1"
}

mkdir -p "$OUT"

start_on explore
take explore 0
act scroll-to 0.36
take hall-of-fame 3

start_on distros
take distros 0
act show-item "'distro:fedora'"
take fedora 6

start_on desktops
take desktops 2

start_on compositors
take compositors 0
act show-item "'compositor:hyprland'"
take hyprland 6

start_on wallpapers 14
take wallpapers 0
act preview-wallpaper 2
take wallpaper-preview 9

start_on system
take system 0
act explain-layers "'desktop:2'"
take fits-together 3

start_on explore
act can-i-run "'compositor:niri'"
take can-i-run 6

flatpak kill "$APP" 2>/dev/null || true
