#!/bin/sh
# Builds and installs the development Flatpak (io.github.omegaui.Syspedia.Devel)
# and runs the Meson tests: cargo tests plus desktop file, GSettings schema and
# metainfo validation. Takes a few minutes; run it in the background.
set -eu

PROJECT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$PROJECT"

exec flatpak run org.flatpak.Builder \
    --user --install --force-clean --ccache \
    _flatpak_build build-aux/io.github.omegaui.Syspedia.Devel.json
