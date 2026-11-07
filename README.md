# Lookbook

**Find a Linux desktop that fits you.**

Lookbook is a friendly guide to the Linux desktop, made for GNOME with GTK 4 and libadwaita. Browse what people run, see how it looks, and find out what your own system can do. It is laid out like an app store, but nothing is sold or installed from it.

- **Explore**: spotlights, collections, setups people are proud of, and the releases of the month.
- **Distros**: a gallery of distributions with real screenshots, filtered by family and release model.
- **Desktops**: GNOME, KDE Plasma and the rest, with popular GNOME extensions read live from extensions.gnome.org.
- **Compositors**: Wayland compositors, the protocols they support, and rices built on them.
- **Wallpapers**: wallpapers you can preview, set or save.
- **Your System**: what you run (distribution, desktop, compositor, hardware) and which ideas, such as a status bar or a lock screen, you can build on it.
- **Can I Run It?**: checks your hardware and your distribution's packages for any desktop or compositor.

Lookbook is free software: no ads, no paywall. It runs on Wayland sessions only.

Lookbook has no release yet. To try it, build it from the source (see below).

## Data that stays current

The catalog of distros, desktops, compositors and rices is published at <https://omegaui.github.io/lookbook/v1/catalog.json>. The app downloads it once a day, so the data stays current between app releases, and falls back to the copy bundled in the app when it is offline. A GitHub Actions workflow refreshes and checks the catalog every day; see [catalog/README.md](catalog/README.md).

Screenshots, wallpapers and rice previews are never bundled. They load live from where their authors host them, and each one is credited:

- Screenshots come from Wikimedia Commons: [docs/screenshot-credits.md](docs/screenshot-credits.md).
- Distro, desktop and compositor logos are trademarks of their owners: [docs/logo-credits.md](docs/logo-credits.md).
- Rices link to their authors' repositories: [docs/rices.md](docs/rices.md).
- Wallpapers are read from their sources and never re-hosted: [docs/research/wallpapers.md](docs/research/wallpapers.md).

## Building

Lookbook builds as a Flatpak. You need `flatpak-builder` (or GNOME Builder), the GNOME 51 SDK and the Rust SDK extension:

```sh
flatpak install --user flathub org.gnome.Sdk//51 org.gnome.Platform//51 org.freedesktop.Sdk.Extension.rust-stable//26.08
```

Build, install and run the development version:

```sh
flatpak-builder --user --install --force-clean _build build-aux/io.github.omegaui.Lookbook.Devel.json
flatpak run io.github.omegaui.Lookbook.Devel
```

`build-aux/io.github.omegaui.Lookbook.json` builds the optimized release version (`io.github.omegaui.Lookbook`).

## Project layout

| Path | Contents |
| --- | --- |
| `src/catalog/` | The catalog model, the bundled copy and the online download |
| `src/system/` | Detection: os-release, desktop session, hardware, Wayland globals |
| `src/pages/`, `src/widgets/` | Pages and widgets, including the isometric artwork |
| `src/news/`, `src/wallpapers/`, `src/extensions.rs`, `src/repology.rs` | Live data: release feeds, wallpaper sources, GNOME extensions, packages |
| `src/ideas.rs`, `src/details.rs`, `src/run_check.rs` | What each idea needs, what a details page compares, and Can I Run It? |
| `data/resources/catalog/catalog.json` | The hand-written catalog, bundled in the app and published online |
| `data/resources/ui/` | Blueprint UI files |
| `data/` | Desktop file, metainfo, GSettings schema and icons |
| `catalog/` | The script that refreshes, checks and publishes the catalog |
| `build-aux/` | Flatpak manifests and build scripts |
| `docs/` | Credits and research notes |
| `po/` | Translations |

## License

Lookbook is licensed under the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE).

The bundled logos keep their own licenses, listed in [docs/logo-credits.md](docs/logo-credits.md). Wayland protocol data in the catalog comes from [wayland-explorer](https://github.com/vially/wayland-explorer) (MIT).
