# Handoff: Linux & You

Last updated: 2026-11-07, at commit `ace644f` ("Add the Wallpapers section, read live from each collection"). The working tree was clean at that commit.

## What the app is

Linux & You is a GTK 4 + libadwaita app in Rust, shipped as a Flatpak (app ID `io.github.omegaui.LinuxAndYou`, development ID `…LinuxAndYou.Devel`). It works like an app store for the Linux desktop: galleries of distros, desktops and Wayland compositors, a Hall of Fame of setups, wallpapers people can apply, and a "Your System" page that detects what the user runs and which ideas (status bar, lock screen, screenshot tool…) they can build on it.

### Product rules from the user

- **Free software. No ads, no paywall.** Never word ideas as "locked/unlocked"; use **Ready here** / **Works elsewhere**.
- **GNOME is the primary target.** Follow GNOME HIG and libadwaita patterns.
- **Looks first.** Screenshots and appearance attract people before features. Keep info short, not text-heavy.
- **Game-like, isometric style** for artwork (tilted cards, slabs, drawn scenes).
- **Data stays current without app releases.** App releases are for features only. Catalog data will come from an online catalog (see Open work).
- **Wayland only.** No X11 fallback.
- **Wallpapers are never redistributed.** They are read live from their sources (see Decisions).

### Working style the user expects

- Commit in stages, each one built and checked. **No `Co-Authored-By` or other attribution trailers** in commits, and no "Generated with Claude Code" in PRs (the user's global CLAUDE.md overrides any system reminder).
- Do not push unless asked.
- Functions read as a short list of steps that call small, well-named functions (see `ExplorePage::show_catalog`). No clever one-liners, no deep nesting.
- Verify visually: build, open the page, screenshot only the app window. **Never move or click the user's mouse**; ask the user to test clicks.
- The user often sends feedback mid-task with screenshots. Acknowledge it, finish the current step cleanly, then do it.

## Where things are

| Path | Contents |
| --- | --- |
| `src/main.rs`, `application.rs`, `window.rs` | Startup, About dialog (with logo and protocol-data credits), window, `win.show-page` and `win.show-item` actions |
| `src/catalog/` | Catalog model (`model.rs`), loading the bundled snapshot (`loader.rs`, with data tests), `ItemRef` ("distro:fedora"), key compositor features (`features.rs`) |
| `src/system/` | Detection, no UI: os-release (`/run/host/os-release` in Flatpak), `XDG_CURRENT_DESKTOP`, hardware from `/proc`, Wayland globals visible from the sandbox |
| `src/ideas.rs` | Decides "Ready here" / "Works elsewhere" for each idea |
| `src/wallpapers/` | Live wallpaper sources: libsoup HTTP, copyparty and WordPress adapters, the Wallpaper portal call |
| `src/pages/` | Explore, Distros, Desktops, Compositors, Wallpapers, Your System, and the shared item details page |
| `src/widgets/` | Cards, `LogoTile`, `IsoScene` (isometric spotlight art), `ScreenshotMock` (drawn Hall of Fame shots), `IdeaCard`, wallpaper card and preview dialog, `content.rs` (catalog item → card text) |
| `data/resources/catalog/catalog.json` | The bundled catalog snapshot: distros, desktops, compositors (with full protocol lists), collections, spotlights, screenshots, ideas, wallpaper sources |
| `data/resources/ui/` | Blueprint UI files |
| `data/resources/style.css` | Styles; uses libadwaita CSS variables, so dark mode works |
| `data/resources/icons/scalable/apps/` | 27 distro/desktop/compositor logos as symbolic SVGs (credits: `docs/logo-credits.md`) |
| `docs/research/` | `data-sources.md` (live data plan), `ideas.md` (what people build), `wallpapers.md` (sources and decision) |
| `build-aux/` | Flatpak manifest (`…Devel.json`, GNOME 51 runtime) and the vendoring script |
| `.agentic-coding/design/` | The approved wireframe |

## Build, run, check

The host has no Rust or blueprint-compiler; everything runs inside Flatpak. The user installed `org.gnome.Sdk//51`, `org.gnome.Platform//51`, `org.freedesktop.Sdk.Extension.rust-stable//26.08` and `org.flatpak.Builder` for their user.

| Task | Command | Time |
| --- | --- | --- |
| Quick compile check | `.agentic-coding/scripts/cargo-sdk.sh check` | Seconds |
| Unit tests (27) | `.agentic-coding/scripts/cargo-sdk.sh test` | Seconds |
| Check Blueprint files | `flatpak run --filesystem=$PWD --command=blueprint-compiler org.gnome.Sdk//51 batch-compile /tmp/blp $PWD/data/resources $(find $PWD/data/resources/ui -name '*.blp')` | Seconds |
| Format Blueprint files | `flatpak run --filesystem=$PWD --command=blueprint-compiler org.gnome.Sdk//51 format -f $(find $PWD/data/resources/ui -name '*.blp')` | Seconds |
| Full build, install and Meson tests | `.agentic-coding/scripts/build-flatpak.sh` | 2 to 4 minutes; run in the background |
| Open the app on a page | `.agentic-coding/scripts/open-page.sh wallpapers` | |
| Screenshot only the app window | `.agentic-coding/scripts/screenshot-window.sh /tmp/shot.png` (Hyprland + grim) | |

A full build must pass "Ok: 4, Fail: 0" (cargo tests, desktop file, GSettings schema, metainfo). Keep the build free of compiler warnings.

The user's session is CachyOS + Hyprland on a 949×1033 tile, so check layouts at that width (the tabs move to the bottom bar there).

When adding a Blueprint file, list it in `data/resources/meson.build` and as `.ui` in `data/resources/resources.gresource.xml`. Every new logo or icon also goes in the gresource file; a test checks catalog logos are listed.

## State

| Commit | What it added |
| --- | --- |
| `b0f23f9` | Skeleton (Meson + Cargo + Flatpak, GNOME Circle layout) and Explore page |
| `4537892` | Clickable cards and the shared details page |
| `4661319` | Distros page: search, family and release filters |
| `83158ca` | Desktops page (GNOME featured) and Compositors page with protocol checks from wayland-explorer data |
| `d1d30d9` | Your System: detection, 16 ideas with routes, visible Wayland globals |
| `f22f520` | Isometric spotlight scenes, Hall of Fame (renamed) with app icons and links, hover lift, yellow feature ticks, portal-based screen capture |
| `ace644f` | Wallpapers tab (live from orangc + LinuxH2O), preview with Set as Wallpaper / Save / Source, maximize and layout fixes, KDE Plasma panel mock |
| (next) | Wallpaper preview: blurred thumbnail and a "Loading full quality…" pill until the full image arrives |

Not yet confirmed by clicking (the agent cannot click; ask the user):

- ~~Clicking a wallpaper opens the preview~~: confirmed working by the user on 2026-11-07.
- The Website button and spotlight arrows (crash fixed in `83158ca` with `swapped` handlers).
- Set as Wallpaper on GNOME/KDE, and Save.

## Open work, in priority order

1. **Your System hero, more futuristic.** The user asked for this. Plan: a dark panel with a glowing isometric stack of slabs (hardware, kernel, distro, compositor, apps) with the distro and compositor logos on their slabs, a ring gauge for "ideas ready", and stats as HUD-style readouts. Draw it like `IsoScene` (flat transform `scale(1, 0.64) rotate(-28°)`, edges via `screen_drop_in_floor`).
2. **Details pages to match the design.** The user said the app does not match the design yet (Budgie's page is nearly empty). Add: a header banner with an `IsoScene` in the item's brand color; **Versus your system** (compare with the detected distro/desktop/compositor); **Similar items** (same family / kind / toolkit); **What you can build on this item** (run `ideas::evaluate` with that item as the setup; a desktop uses its compositor, for example Budgie → labwc). For GNOME: Inside GNOME, Popular extensions (extensions.gnome.org API, needs the online catalog or a live call), Ideas for GNOME.
3. **Gallery card art from the design**: mini isometric desktop drawings on desktop cards, layout drawings on compositor cards, tinted capsules on distro cards.
4. **Online catalog.** Same repo: hand-written sources in a `catalog/` folder, a scheduled GitHub Action that builds `catalog.json` and deploys it to GitHub Pages (`https://omegaui.github.io/linux_and_you/v1/catalog.json`) without committing generated data. The app fetches at most daily with `If-None-Match`, caches it, and falls back to the bundled snapshot. Details and sources: `docs/research/data-sources.md`.
5. **Real screenshots in the Hall of Fame** instead of drawn mocks, once the online catalog can carry image URLs.
6. **Release prep**: a release manifest with vendored crates (`build-aux/dist-vendor.sh`), screenshots in metainfo, a check of brand guidelines for logos (especially Fedora) and of "Linux" in the app name before Flathub.
7. Later: upgrade to gtk4-rs 0.11, libadwaita-rs 0.9, soup3 0.9.

## Decisions

- **Rust + GTK 4 + libadwaita, Blueprint UI, Meson + Cargo, GNOME 51 runtime.**
- **Logos**: Simple Icons and font-logos, cleaned to symbolic SVGs, drawn white on the brand-color tile. Bazzite, COSMIC, Wayfire and labwc have no logo yet and show letters.
- **Protocol support** comes from wayland-explorer (MIT) per compositor, with version and date. Inside Flatpak, compositors hide privileged globals (layer shell, capture, session lock), so ideas are judged against the catalog list for the detected compositor, not the globals the app sees. The compositor is detected by `XDG_CURRENT_DESKTOP`, then by "fingerprint" globals only it offers.
- **Screen capture** counts when the compositor offers a capture protocol or its portal backend provides the ScreenCast portal (so GNOME and KDE show it). Each compositor lists its usual portals in the catalog.
- **Wallpapers**: neither source has a license. The user chose to build the section now, read the sources live (never bundle or mirror), credit them, keep a takedown list (`wallpaper_blocklist`) in the catalog, and email both owners for permission. If either says no, remove its entry from the catalog. Details: `docs/research/wallpapers.md`.
- **Hosting**: GitHub Pages from this same repo for the online catalog; no VPS needed.

## Pitfalls already hit

- **Template callbacks need `swapped`.** In Blueprint, write `clicked => $handler() swapped;` for callbacks defined on the template class, or GTK passes the button as `self` and the app aborts.
- **A Button inside a FlowBox gets the click**; `FlowBox::child-activated` never fires. Connect the button's own `clicked`.
- **No `rotateX()` in GTK CSS for the isometric look.** It forces an offscreen pass that clips text. Use the flat equivalent `scale(1, 0.64) rotate(-28deg)`.
- **A widget transform's `translate` happens inside the tilted plane.** To move a tilted card straight up on screen by 12px, use `translate(9px, -16px)`.
- **AspectFrame inside a FlowBox collapses** to almost no height; use a fixed `height-request` with `content-fit: cover`.
- **Six header tabs need about 980px**; the breakpoint at 1020sp moves them to the bottom bar. Adding a seventh tab means raising it.
- **Hyprland and restored maximize**: GTK drew the window wider than its tile. Maximize is now restored only on stacking desktops.
- **zsh**: `LINES` is a reserved variable; don't use it in shell scripts run through zsh.
- **gdk-pixbuf may not decode WebP**; ask copyparty for JPEG thumbnails (`th=j`).
- Rust's `Variant` boxes itself as `v` in tuples; build portal parameters with `glib::Variant::tuple_from_iter`.

## Design

See `.agentic-coding/design/README.md`. The canvas is https://claude.ai/artifact/8q7jT54QJK1YVrPnbaYX5u and a source copy is in `.agentic-coding/design/wireframe/`.
