# Design: store-style wireframe

The approved design lives in a Claude Design canvas:

**https://claude.ai/artifact/8q7jT54QJK1YVrPnbaYX5u** (private to the owner, omegaui)

`wireframe/` holds a copy of the canvas source as of 2026-11-07, so the design survives even without access to the link:

| File | Screen |
| --- | --- |
| `canvas.json` | Canvas layout: artboard positions, titles and sticky notes |
| `Main.dc.html` | Explore (home): tilted collection strip, spotlight, Hall of fame, distro/desktop/compositor rows, Your system banner |
| `Distros.dc.html` | Distro gallery with search and filters |
| `Distro.dc.html` | Distro details (Fedora): Hall of fame, At a glance, Versus your system, Similar distros |
| `Desktops.dc.html` | Desktop gallery with a GNOME banner and mini isometric desktop drawings on cards |
| `Desktop.dc.html` | GNOME details: Hall of fame, Inside GNOME, Popular extensions, Ideas for GNOME |
| `YourSystemGnome.dc.html` | Your System on a GNOME session: isometric stack, stats, ideas with routes, what GNOME offers |
| `Compositors.dc.html` | Compositor gallery with isometric layout drawings and protocol checks |
| `YourSystem.dc.html` | Your System on a Hyprland session |

Each `.dc.html` file is one artboard written for the Design canvas runtime (`<x-dc>` markup, `{{holes}}`, a `DCLogic` class). They need that runtime to render; read them as source for layout, copy and data.

The design was made before these later decisions, which take precedence over it:

- "Wall of fame" is now **Hall of Fame**.
- Ideas say **Ready here** / **Works elsewhere**, never "locked": the app is free, with no ads and nothing paid.
- A **Wallpapers** tab exists (not in the wireframe).
- Sample data in the wireframe (versions, hardware, protocol support) is not authoritative; the app's catalog is.
