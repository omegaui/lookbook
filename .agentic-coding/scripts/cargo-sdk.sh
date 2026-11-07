#!/bin/sh
# Runs cargo inside the GNOME 51 SDK with its Rust extension. Use it for quick
# checks; it takes seconds, where a full Flatpak build takes minutes.
#
#   .agentic-coding/scripts/cargo-sdk.sh check
#   .agentic-coding/scripts/cargo-sdk.sh test
#
# Meson normally generates src/config.rs (it is gitignored). When it is missing,
# this script writes a development copy so cargo can compile.
set -eu

PROJECT=$(cd "$(dirname "$0")/../.." && pwd)
CACHE="$PROJECT/.cargo-sdk"
mkdir -p "$CACHE"

if [ ! -f "$PROJECT/src/config.rs" ]; then
    cat > "$PROJECT/src/config.rs" <<'CONFIG'
pub const APP_ID: &str = "io.github.omegaui.LinuxAndYou.Devel";
pub const GETTEXT_PACKAGE: &str = "linux-and-you";
pub const LOCALEDIR: &str = "/app/share/locale";
pub const PROFILE: &str = "Devel";
pub const RESOURCES_FILE: &str = concat!("/app/share/linux-and-you", "/resources.gresource");
pub const VERSION: &str = "0.1.0-devel";
CONFIG
fi

# CARGO_BUILD_JOBS stays at 8: this machine has 28 cores and more jobs can freeze it.
exec flatpak run \
    --share=network \
    --filesystem="$PROJECT" \
    --env=CARGO_HOME="$CACHE/home" \
    --env=CARGO_TARGET_DIR="$CACHE/target" \
    --env=CARGO_BUILD_JOBS=8 \
    --env=PATH=/usr/bin:/usr/lib/sdk/rust-stable/bin \
    --command=cargo \
    org.gnome.Sdk//51 "$@" --manifest-path "$PROJECT/Cargo.toml"
