#!/bin/sh
# Scrolls the page in view of the running development build, without the mouse.
# The argument is a fraction of the page height: 0 is the top, 1 the bottom.
#
#   .agentic-coding/scripts/scroll-to.sh 0.5
set -eu

APP=io.github.omegaui.Syspedia.Devel
FRACTION=${1:?usage: scroll-to.sh fraction}

flatpak run --command=gapplication "$APP" action "$APP" scroll-to "$(printf %f "$FRACTION")"
echo "Scrolled to $FRACTION"
