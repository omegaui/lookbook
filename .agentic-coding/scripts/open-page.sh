#!/bin/sh
# Restarts the development build on one page, so it can be screenshotted
# without clicking. Pages: explore, distros, desktops, compositors, wallpapers, system.
#
#   .agentic-coding/scripts/open-page.sh wallpapers
set -eu

APP=io.github.omegaui.LinuxAndYou.Devel
PAGE=${1:-explore}

flatpak kill "$APP" 2>/dev/null || true
flatpak run --command=gsettings "$APP" set "$APP" last-page "$PAGE"
flatpak run "$APP" >/dev/null 2>&1 &
echo "Opened $APP on $PAGE"
