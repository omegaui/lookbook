#!/bin/sh
# Builds the release app and packs it into a single .flatpak file to attach
# to a GitHub release. People install it with
#
#   flatpak install --user syspedia-VERSION-ARCH.flatpak
#
# and Flatpak fetches the GNOME runtime from Flathub, which the bundle names.
#
#   build-aux/make-bundle.sh            # writes syspedia-<version>-<arch>.flatpak
set -eu

PROJECT=$(cd "$(dirname "$0")/.." && pwd)
cd "$PROJECT"

APP_ID=io.github.omegaui.Syspedia
VERSION=$(sed -n "s/^  version: '\(.*\)',$/\1/p" meson.build)
ARCH=$(flatpak --default-arch)
BUNDLE="syspedia-$VERSION-$ARCH.flatpak"

flatpak run org.flatpak.Builder --force-clean --ccache \
    --state-dir=.flatpak-builder-release \
    --repo=_flatpak_repo \
    _flatpak_build_release build-aux/$APP_ID.json

flatpak build-bundle \
    --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo \
    _flatpak_repo "$BUNDLE" "$APP_ID" master

echo "Wrote $BUNDLE"
