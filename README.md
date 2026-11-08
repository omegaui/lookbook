# Syspedia

**Find a Linux desktop that fits you.**

Syspedia is a friendly guide to the Linux desktop, made for GNOME with GTK 4 and libadwaita. Browse what people run, see how it looks, and find out what your own system can do. It is laid out like an app store, but nothing is sold or installed from it.

- **Explore**: spotlights, collections, the releases of the month, and a Hall of Fame of Hyprland's ricing contest winners.
- **Distros**: a hand-picked gallery of distributions, filtered by family and release model, each with a page of real screenshots.
- **Desktops**: GNOME, KDE Plasma and the rest, with popular GNOME extensions read live from extensions.gnome.org.
- **Compositors**: Wayland compositors, the protocols they support, and rices built on them.
- **Wallpapers**: wallpapers you can preview, set or save.
- **Your System**: what you run (distribution, desktop, compositor, hardware) and which ideas, such as a status bar or a lock screen, you can build on it.
- **How Linux Fits Together**: what a distribution, a desktop and a compositor each do, drawn as layers you can click.
- **Can I Run It?**: checks your hardware and your distribution's packages for any desktop or compositor.

Syspedia is free software: no ads, no paywall. It runs on Wayland sessions only.

## Install

Syspedia is not on Flathub. Its page is **<https://omegaui.github.io/syspedia>**. Install or update it with one command; it needs only Flatpak and installs for your user, with no sudo:

```sh
curl -sL https://omegaui.github.io/syspedia/install.sh | sh
```

Run the same command again to update; it does nothing when you already have the latest version. Add `-s -- --source` after `sh` to build the latest release from source instead, or `-s -- --uninstall` to remove Syspedia. The first install also fetches the GNOME runtime from Flathub (about 1 GB, shared with other GNOME apps).

To install by hand, download `syspedia-x86_64.flatpak` from the [latest release](https://github.com/omegaui/syspedia/releases/latest), then:

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user syspedia-x86_64.flatpak
```

## Data that stays current

The catalog of distros, desktops, compositors and rices is published at <https://omegaui.github.io/syspedia/v1/catalog.json>. The app downloads it once a day, so the data stays current between app releases, and falls back to the copy bundled in the app when it is offline. A GitHub Actions workflow refreshes and checks the catalog every day; see [catalog/README.md](catalog/README.md).

Screenshots, wallpapers and rice previews are never bundled. They load live from where their authors host them, and each one is credited:

- Screenshots come mostly from Wikimedia Commons, the rest from each project's own site or repository, or a rice's README: [docs/screenshot-credits.md](docs/screenshot-credits.md).
- Hall of Fame screenshots come from [Hyprland's Hall of Fame](https://hypr.land/hall_of_fame/) and belong to each rice's author.
- Distro, desktop and compositor logos are trademarks of their owners: [docs/logo-credits.md](docs/logo-credits.md).
- Rices link to their authors' repositories: [docs/rices.md](docs/rices.md).
- Wallpapers are read from their sources and never re-hosted: [docs/research/wallpapers.md](docs/research/wallpapers.md).

## Build from source

Syspedia builds as a Flatpak, so the build needs nothing installed on your system except Flatpak itself. Install flatpak-builder, the GNOME 51 SDK and the Rust SDK extension from Flathub:

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user flathub org.flatpak.Builder org.gnome.Sdk//51 org.gnome.Platform//51 org.freedesktop.Sdk.Extension.rust-stable//26.08
```

Get the source, then build and install the release version (it downloads the Rust crates during the build and takes a few minutes):

```sh
git clone https://github.com/omegaui/syspedia.git
cd syspedia
flatpak run org.flatpak.Builder --user --install --force-clean _build build-aux/io.github.omegaui.Syspedia.json
flatpak run io.github.omegaui.Syspedia
```

To build a specific release, check out its tag first, for example `git checkout v0.1.0`.

For development, `build-aux/io.github.omegaui.Syspedia.Devel.json` builds `io.github.omegaui.Syspedia.Devel`, which installs next to the release version and has a striped header bar. `build-aux/make-bundle.sh` builds the `.flatpak` bundle attached to releases.

## Project layout

| Path | Contents |
| --- | --- |
| `src/catalog/` | The catalog model, the bundled copy and the online download |
| `src/system/` | Detection: os-release, desktop session, hardware, Wayland globals |
| `src/pages/`, `src/widgets/` | Pages and widgets |
| `src/news/`, `src/wallpapers/`, `src/extensions.rs`, `src/repology.rs` | Live data: release feeds, wallpaper sources, GNOME extensions, packages |
| `src/ideas.rs`, `src/details.rs`, `src/run_check.rs` | What each idea needs, what a details page compares, and Can I Run It? |
| `src/fits_together.rs` | How Linux Fits Together and the page intros |
| `data/resources/catalog/catalog.json` | The hand-written catalog, bundled in the app and published online |
| `data/resources/ui/` | Blueprint UI files |
| `data/` | Desktop file, metainfo, GSettings schema and icons |
| `catalog/` | The scripts that refresh, check and publish the catalog, and write the screenshot credits |
| `build-aux/` | Flatpak manifests and build scripts |
| `docs/` | Credits and research notes |
| `po/` | Translations |

## License

Syspedia is licensed under the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE).

The bundled logos keep their own licenses, listed in [docs/logo-credits.md](docs/logo-credits.md). Wayland protocol data in the catalog comes from [wayland-explorer](https://github.com/vially/wayland-explorer) (MIT).
