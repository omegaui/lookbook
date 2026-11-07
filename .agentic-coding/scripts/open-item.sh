#!/bin/sh
# Opens a details page in the running development build, without clicking.
# Items look like "distro:fedora", "desktop:gnome" or "compositor:niri".
#
#   .agentic-coding/scripts/open-item.sh desktop:gnome
set -eu

APP=io.github.omegaui.Lookbook.Devel
ITEM=${1:?usage: open-item.sh kind:id}

flatpak run --command=gapplication "$APP" action "$APP" show-item "'$ITEM'"
echo "Opened $ITEM"
