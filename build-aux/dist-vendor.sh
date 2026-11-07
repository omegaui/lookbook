#!/bin/sh
# Meson runs this as "/bin/sh dist-vendor.sh DIST SOURCE_ROOT",
# so it must stay POSIX sh.
set -eu

export DIST="$1"
export SOURCE_ROOT="$2"

cd "$SOURCE_ROOT"
mkdir "$DIST"/.cargo

# Vendor the crates so the release tarball builds offline.
cargo vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > "$DIST"/.cargo/config.toml

# Put the vendored crates inside the dist tarball.
mv vendor "$DIST"
