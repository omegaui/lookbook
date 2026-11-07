#!/bin/sh
# Turns the Explore spotlight carousel of the running development build to a
# page, counting from 0, without the mouse.
#
#   .agentic-coding/scripts/show-spotlight.sh 3
set -eu

APP=io.github.omegaui.Lookbook.Devel
INDEX=${1:?usage: show-spotlight.sh index}

flatpak run --command=gapplication "$APP" action "$APP" show-spotlight "int32 $INDEX"
echo "Showing spotlight $INDEX"
