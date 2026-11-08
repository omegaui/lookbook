#!/bin/sh
# Installs Syspedia, or updates it to the latest release.
#
#   curl -sL https://omegaui.github.io/syspedia/install.sh | sh
#
# Run the same command again to update; it does nothing when Syspedia is
# already up to date. Options go after "sh -s --":
#
#   curl -sL https://omegaui.github.io/syspedia/install.sh | sh -s -- --source
#   curl -sL https://omegaui.github.io/syspedia/install.sh | sh -s -- --uninstall
#
#   --source     build the latest release from source instead of using the
#                ready-made bundle (takes a while; needs no other tools)
#   --uninstall  remove Syspedia (your settings stay in ~/.var/app)
#
# Everything goes into your user's Flatpak installation; nothing needs sudo.
set -eu

APP_ID=io.github.omegaui.Syspedia
REPO=omegaui/syspedia
RELEASES=https://github.com/$REPO/releases
FLATHUB=https://dl.flathub.org/repo/flathub.flatpakrepo
BUNDLE=syspedia-x86_64.flatpak

say() {
    printf '%s\n' "$*"
}

fail() {
    printf 'Syspedia: %s\n' "$*" >&2
    exit 1
}

main() {
    mode=bundle
    for argument in "$@"; do
        case "$argument" in
            --source) mode=source ;;
            --uninstall) mode=uninstall ;;
            -h | --help) usage; exit 0 ;;
            *) fail "unknown option $argument (try --help)" ;;
        esac
    done

    command -v flatpak >/dev/null 2>&1 || fail "Flatpak is not installed. See https://flatpak.org/setup/ for your distribution."

    if [ "$mode" = uninstall ]; then
        uninstall
        return
    fi

    warn_without_wayland
    latest=$(latest_version)
    installed=$(installed_version)
    if [ "$installed" = "$latest" ]; then
        say "Syspedia $installed is installed and up to date."
        return
    fi

    add_flathub
    if [ "$mode" = source ]; then
        install_from_source "$latest"
    else
        install_bundle
    fi
    report "$installed" "$latest"
}

usage() {
    cat <<'USAGE'
Installs Syspedia, or updates it to the latest release.

  curl -sL https://omegaui.github.io/syspedia/install.sh | sh
  curl -sL https://omegaui.github.io/syspedia/install.sh | sh -s -- --source
  curl -sL https://omegaui.github.io/syspedia/install.sh | sh -s -- --uninstall

  --source     build the latest release from source instead of the bundle
  --uninstall  remove Syspedia (your settings stay in ~/.var/app)
USAGE
}

# The newest release's version, read from where GitHub's "latest" link leads.
latest_version() {
    location=$(curl -fsSIL -o /dev/null -w '%{url_effective}' "$RELEASES/latest") ||
        fail "could not reach GitHub. Check your connection."
    case "$location" in
        */releases/tag/*) ;;
        *) fail "there is no release yet. See $RELEASES." ;;
    esac
    tag=${location##*/}
    printf '%s\n' "${tag#v}"
}

# The installed version, or nothing when Syspedia is not installed.
installed_version() {
    flatpak info --user "$APP_ID" 2>/dev/null | sed -n 's/^ *Version: *//p' | head -n 1
}

warn_without_wayland() {
    if [ "${XDG_SESSION_TYPE:-}" = x11 ]; then
        say "Note: Syspedia needs a Wayland session; it will not open on X11."
    fi
}

# Flatpak fetches the GNOME runtime Syspedia runs on from Flathub.
add_flathub() {
    flatpak remote-add --user --if-not-exists flathub "$FLATHUB"
}

install_bundle() {
    case "$(uname -m)" in
        x86_64) ;;
        *) fail "the ready-made bundle is for x86_64 only. Run again with --source to build it." ;;
    esac
    folder=$(mktemp -d)
    trap 'rm -rf "$folder"' EXIT
    say "Downloading Syspedia…"
    curl -fL --progress-bar -o "$folder/$BUNDLE" "$RELEASES/latest/download/$BUNDLE" ||
        fail "could not download $BUNDLE from the latest release."
    say "Installing…"
    flatpak install --user --noninteractive --reinstall "$folder/$BUNDLE"
}

# Builds the release's source with flatpak-builder, itself a Flatpak, so
# nothing but Flatpak is needed. The first build downloads the GNOME SDK.
install_from_source() {
    version=$1
    folder=$(mktemp -d)
    trap 'rm -rf "$folder"' EXIT
    say "Installing the build tools from Flathub…"
    flatpak install --user --noninteractive flathub \
        org.flatpak.Builder org.gnome.Sdk//51 org.gnome.Platform//51 \
        org.freedesktop.Sdk.Extension.rust-stable//26.08
    say "Downloading Syspedia $version's source…"
    curl -fsSL "https://github.com/$REPO/archive/refs/tags/v$version.tar.gz" | tar -xz -C "$folder" ||
        fail "could not download the source of version $version."
    say "Building (this takes several minutes)…"
    (
        cd "$folder"/syspedia-*
        flatpak run org.flatpak.Builder --user --install --force-clean \
            _build "build-aux/$APP_ID.json"
    )
}

uninstall() {
    if [ -z "$(installed_version)" ]; then
        say "Syspedia is not installed."
        return
    fi
    flatpak uninstall --user --noninteractive "$APP_ID"
    say "Syspedia is removed. Your settings stay in ~/.var/app/$APP_ID."
}

report() {
    if [ -z "$1" ]; then
        say "Syspedia $2 is installed. Find it in your app launcher, or run: flatpak run $APP_ID"
    else
        say "Syspedia is updated from $1 to $2."
    fi
}

main "$@"
