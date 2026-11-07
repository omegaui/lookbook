#!/bin/sh
# Builds the optimized release profile (io.github.omegaui.Lookbook) and
# installs it for the current user, next to the development build. The crates
# are downloaded during the build; a Flathub submission needs them vendored
# instead (build-aux/dist-vendor.sh). Takes several minutes.
set -eu

PROJECT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$PROJECT"

exec flatpak run org.flatpak.Builder \
    --user --install --force-clean --ccache \
    --state-dir=.flatpak-builder-release \
    _flatpak_build_release build-aux/io.github.omegaui.Lookbook.json
