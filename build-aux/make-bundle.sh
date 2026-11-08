#!/bin/sh
# Builds the release app and packs it into a single .flatpak file to attach
# to a GitHub release. People install it with
#
#   flatpak install --user syspedia-ARCH.flatpak
#
# and Flatpak fetches the GNOME runtime from Flathub, which the bundle names.
# The name has no version, so the install script can always download
# releases/latest/download/syspedia-x86_64.flatpak.
#
#   build-aux/make-bundle.sh            # writes syspedia-<arch>.flatpak
set -eu

PROJECT=$(cd "$(dirname "$0")/.." && pwd)
cd "$PROJECT"

APP_ID=io.github.omegaui.Syspedia
ARCH=$(flatpak --default-arch)
BUNDLE="syspedia-$ARCH.flatpak"

flatpak run org.flatpak.Builder --force-clean --ccache \
    --state-dir=.flatpak-builder-release \
    --repo=_flatpak_repo \
    _flatpak_build_release build-aux/$APP_ID.json

flatpak build-bundle \
    --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo \
    _flatpak_repo "$BUNDLE" "$APP_ID" master

echo "Wrote $BUNDLE"
