# Linux & You

A friendly guide to the Linux desktop, built with GTK 4 and libadwaita.

- **Distros**: a gallery of Linux distributions.
- **Compositors**: a gallery of Wayland compositors and the protocols they support.
- **Your System**: your distribution, session, compositor, hardware, and the tools you can build on your compositor.

Linux & You targets Wayland sessions only.

## Building

The app builds as a Flatpak with GNOME Builder or `flatpak-builder`:

```sh
flatpak-builder --user --install --force-clean _build build-aux/io.github.omegaui.LinuxAndYou.Devel.json
flatpak run io.github.omegaui.LinuxAndYou.Devel
```

## Project layout

| Path | Contents |
| --- | --- |
| `build-aux/` | Flatpak manifests and build scripts |
| `data/` | Desktop file, metainfo, GSettings schema, icons |
| `data/resources/catalog/` | Bundled JSON for distros, compositors, protocols and ideas |
| `data/resources/ui/` | Blueprint UI files |
| `docs/wireframes/` | UI wireframes |
| `po/` | Translations |
| `src/system/` | Host detection: os-release, session, Wayland globals, hardware |
| `src/catalog/` | Loads the bundled catalog |
| `src/models/` | GObject models for list views |
| `src/pages/`, `src/widgets/` | UI |

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
