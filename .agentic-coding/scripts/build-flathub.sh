#!/bin/sh
# Builds the Flathub manifest the way Flathub does (no network during the
# build), but from this repository's HEAD instead of GitHub, and does not
# install it. Takes several minutes.
#
#   .agentic-coding/scripts/build-flathub.sh
set -eu

PROJECT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$PROJECT"
MANIFEST=build-aux/flathub/local-test.json

python3 - "$MANIFEST" <<'PYTHON'
import json, subprocess, sys
manifest = json.load(open("build-aux/flathub/io.github.omegaui.Syspedia.json"))
head = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
manifest["modules"][0]["sources"][0] = {"type": "git", "path": "../..", "commit": head}
json.dump(manifest, open(sys.argv[1], "w"), indent=4)
PYTHON

trap 'rm -f "$MANIFEST"' EXIT
flatpak run org.flatpak.Builder --user --force-clean --ccache \
    --state-dir=.flatpak-builder-flathub \
    _flatpak_build_flathub "$MANIFEST"
