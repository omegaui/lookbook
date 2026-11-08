#!/bin/sh
# Opens "How Linux Fits Together" in the running development build on a setup
# (yours, desktop or compositor) and a layer counted from the top, without the mouse.
#
#   .agentic-coding/scripts/explain-layers.sh desktop:2
set -eu

APP=io.github.omegaui.Syspedia.Devel
TARGET=${1:-yours:0}

flatpak run --command=gapplication "$APP" action "$APP" explain-layers "'$TARGET'"
echo "Explaining $TARGET"
