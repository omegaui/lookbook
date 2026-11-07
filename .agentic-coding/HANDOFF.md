# Handoff: Lookbook

Last updated: 2026-11-07, after the rename to Lookbook. Check `git log` for anything newer.

The app was called **Linux & You** until 2026-11-07, when the user renamed it **Lookbook** (see Decisions). **Current focus: the review** in `review.md` (repo root, gitignored, local only): Flathub blockers, bugs and half-done work, worked through item by item (see Open work, item 2).

## What the app is

Lookbook is a GTK 4 + libadwaita app in Rust, shipped as a Flatpak (app ID `io.github.omegaui.Lookbook`, development ID `…Lookbook.Devel`, shown as "Lookbook (Devel)" in launchers). It is a friendly guide to the Linux desktop, laid out like a store (think Bazaar or Flathub) but not one: nothing is sold or installed from it. It has galleries of distros, desktops and Wayland compositors (each details page has a Screenshots gallery), a Hall of Fame of Hyprland's ricing contest winners, wallpapers people can apply, and a "Your System" page that detects what the user runs and which ideas (status bar, lock screen, screenshot tool…) they can build on it.

### Product rules from the user

- **Free software. No ads, no paywall.** Never word ideas as "locked/unlocked"; use **Ready here** / **Works elsewhere**.
- **GNOME is the primary target.** Follow GNOME HIG and libadwaita patterns.
- **Looks first.** Screenshots and appearance attract people before features. Keep info short, not text-heavy.
- **Flat, no isometric art** (changed 2026-11-07; the user found tilted cards and screenshots unappealing). Banner screenshots sit straight on and run off the banner's right and bottom edges, like GNOME Software's featured banners (`EdgePicture`, `edge.rs`). Drawn art is flat too.
- **Screenshots, not a Hall of Fame, on details pages.** Every distro, desktop and compositor details page has a Screenshots gallery of real, good-looking screenshots. Explore's Hall of Fame shows the real winners of Hyprland's ricing contests.
- **Data stays current without app releases.** App releases are for features only. The app reads the online catalog published by the Catalog workflow (see `catalog/README.md`).
- **Wayland only.** No X11 fallback.
- **Wallpapers are never redistributed.** They are read live from their sources (see Decisions).

### Working style the user expects

- Commit in stages, each one built and checked. **No `Co-Authored-By` or other attribution trailers** in commits, and no "Generated with Claude Code" in PRs (the user's global CLAUDE.md overrides any system reminder).
- The repo is public on GitHub (`origin`, https://github.com/omegaui/lookbook; it was `linux_and_you` before the rename, and the local folder is still `~/Projects/linux_and_you`). Push only when the user asks; they have asked after each finished stage so far, and sometimes push themselves (run `git fetch` before assuming what is on `origin`).
- Functions read as a short list of steps that call small, well-named functions (see `ExplorePage::show_catalog`). No clever one-liners, no deep nesting.
- Verify visually: build, open the page, screenshot only the app window. **Never move or click the user's mouse**; ask the user to test clicks.
- The user often sends feedback mid-task with screenshots. Acknowledge it, finish the current step cleanly, then do it.

## Where things are

| Path | Contents |
| --- | --- |
| `src/main.rs`, `application.rs`, `window.rs` | Startup, About dialog (credits for logos, screenshots, Hall of Fame, Repology and protocol data; catalog date), Keyboard Shortcuts dialog and accelerators, window (toasts, Refresh Catalog), `win.show-page`, `win.show-item`, `win.can-i-run` (also app actions), `win.fits-together`, `win.what-is-lookbook`, and development-only `scroll-to` / `show-spotlight` / `explain-layers` |
| `src/catalog/` | Catalog model (`model.rs`), the bundled snapshot (`loader.rs`, with data tests), the downloaded online catalog (`online.rs`), `ItemRef` ("distro:fedora"), key compositor features (`features.rs`) |
| `src/system/` | Detection, no UI: os-release (`/run/host/os-release` in Flatpak), `XDG_CURRENT_DESKTOP`, hardware from `/proc`, Wayland globals visible from the sandbox |
| `src/ideas.rs` | Decides "Ready here" / "Works elsewhere" for each idea |
| `src/fits_together.rs` | The layers and plain-language explanations for "How Linux Fits Together". No UI code |
| `src/details.rs` | What a details page compares (Versus your system), which ideas an item supports, and similar items. No UI code |
| `src/extensions.rs` | Popular GNOME Shell extensions, read live from the extensions.gnome.org API |
| `src/run_check.rs`, `src/repology.rs` | "Can I Run It?": hardware checks, ideas gained or given up, ways to try, and the Repology package lookup (HTTP/1.1, at most one request a second) |
| `src/news/` | "What's new this month": feed parsers (kernel.org, endoflife.date, Atom, RSS), dates, and picking recent releases |
| `src/http.rs` | The libsoup session (User-Agent with a contact link), a shared session, GET helpers |
| `src/wallpapers/` | Live wallpaper sources: libsoup HTTP, copyparty and WordPress adapters, the Wallpaper portal call |
| `src/pages/` | Explore, Distros, Desktops, Compositors, Wallpapers, Your System, and the shared item details page |
| `src/widgets/` | Cards (`CatalogCard`, `ShowcaseCard` for the Distros and Desktops galleries, `RiceCard`, release card, extension card, `IdeaCard`, `ScreenshotCard` for Hall of Fame winners), `LogoTile`, `IconTile` (protocol cards), banner art (`EdgePicture`, `DesktopScene` drawn fallback, both placed by `edge.rs`), `MiniDesktop`, `LayerStack` (Your System), `ScreenshotGallery` (details pages), `ScreenshotPicture` (16:10 frame, cover or whole image), `CoverPicture` (fixed-height cover image or video), spotlight cards (`SpotlightCard`, `WallpaperSpotlight`), dialogs (`RunCheckDialog`, `WallpaperDialog`), `NaturalWidthCap`, `content.rs` (catalog item → card text) |
| `data/resources/catalog/catalog.json` | The catalog: distros, desktops, compositors (with protocol lists, a main screenshot and a `gallery`), collections, spotlights, Hall of Fame (`hall_of_fame`, refreshed from the Hyprland website), ideas (with `built_into`), protocols, wallpaper sources, release feeds (`news_sources`), rices. Bundled in the app and published online |
| `catalog/` | The publishing pipeline: `build.py` (refresh, check, publish), `test_build.py`, `sources.json`, `README.md` |
| `.github/workflows/catalog.yml` | Builds and deploys the catalog to GitHub Pages daily, on catalog changes and by hand |
| `data/icons/` | The app icon as PNGs in `hicolor/<size>/apps/` (a penguin coding at a desk), its Devel variant and the symbolic SVG, all generated by `.agentic-coding/design/icon/make_icon.py` from `source/penguin-at-desk.png` |
| `data/resources/ui/` | Blueprint UI files |
| `data/resources/style.css` | Styles; uses libadwaita CSS variables, so dark mode works |
| `data/resources/icons/scalable/apps/` | 31 distro/desktop/compositor logos (and Tux for kernel releases) as symbolic SVGs (credits: `docs/logo-credits.md`) |
| `docs/` | `logo-credits.md`, `screenshot-credits.md`, `rices.md`; `research/` holds `data-sources.md` (live data plan), `ideas.md` (what people build), `wallpapers.md` (sources and decision) |
| `build-aux/` | Flatpak manifests for the release (`io.github.omegaui.Lookbook.json`) and development (`….Devel.json`) builds, GNOME 51 runtime, and the vendoring script |
| `.agentic-coding/design/` | The approved wireframe |

## Build, run, check

The host has no Rust or blueprint-compiler; everything runs inside Flatpak. The user installed `org.gnome.Sdk//51`, `org.gnome.Platform//51`, `org.freedesktop.Sdk.Extension.rust-stable//26.08` and `org.flatpak.Builder` for their user.

| Task | Command | Time |
| --- | --- | --- |
| Quick compile check | `.agentic-coding/scripts/cargo-sdk.sh check` | Seconds |
| Unit tests | `.agentic-coding/scripts/cargo-sdk.sh test` | Seconds |
| Check Blueprint files | `flatpak run --filesystem=$PWD --command=blueprint-compiler org.gnome.Sdk//51 batch-compile /tmp/blp $PWD/data/resources $(find $PWD/data/resources/ui -name '*.blp')` | Seconds |
| Format Blueprint files | `flatpak run --filesystem=$PWD --command=blueprint-compiler org.gnome.Sdk//51 format -f $(find $PWD/data/resources/ui -name '*.blp')` | Seconds |
| Full build, install and Meson tests | `.agentic-coding/scripts/build-flatpak.sh` | 2 to 4 minutes; run in the background |
| Build and install the optimized release app (`io.github.omegaui.Lookbook`, next to the Devel one) | `.agentic-coding/scripts/install-release.sh` | About 5 minutes |
| Open the app on a page | `.agentic-coding/scripts/open-page.sh wallpapers` | |
| Open a details page in the running app | `.agentic-coding/scripts/open-item.sh desktop:gnome` | |
| Scroll the page in view (development builds only) | `.agentic-coding/scripts/scroll-to.sh 0.5` (0 is the top, 1 the bottom) | |
| Turn the Explore spotlight carousel (development builds only) | `.agentic-coding/scripts/show-spotlight.sh 3` (counts from 0) | |
| Open What Is Lookbook?, Keyboard Shortcuts or Refresh Catalog without the mouse | `flatpak run --command=gapplication io.github.omegaui.Lookbook.Devel action io.github.omegaui.Lookbook.Devel what-is-lookbook` (or `shortcuts`, `refresh-catalog`; the first and last are development builds only) | |
| Open "How Linux Fits Together" on a setup and layer (development builds only) | `.agentic-coding/scripts/explain-layers.sh desktop:2` (setups: yours, desktop, compositor; layers count from the top) | |
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
| `fe27f7f` | Wallpaper preview: blurred thumbnail and a "Loading full quality…" pill until the full image arrives |
| `da79927` | Your System redesign: glowing isometric stack (`IsoStack`), Scan Again / Copy Report, GPU names from `pci.ids`, idea icon tiles, protocol cards with descriptions (`catalog.protocols`, `ProtocolCard`, `IsoTile`) on Your System and compositor details |
| `5d2bcf7` | Details pages and Desktops gallery to match the design: brand-color banner, Hall of Fame, Inside/At a Glance, Popular Extensions (GNOME, live), What You Can Build, Versus your system, Similar items; GNOME banner, toolkit filter; Bazzite, labwc and Wayfire logos |
| `b37ae41` | Four spotlights: Bazzite, GNOME, niri, and a live wallpaper |
| `33384f0` | Can I Run It? (hardware, Repology packages, ideas gained or given up, ways to try), Built in ideas, real desktop screenshots from Wikimedia Commons |
| `7217cf5` | What's New This Month on Explore, from release feeds |
| `597dd65` | Real screenshots for distros, compositors, details banners, spotlights and the Hall of Fame; Distros page uses `ShowcaseCard` |
| `4e563a4` | Rices on the Compositors page and compositor details |
| `6b84c7e` | Online catalog: Catalog workflow, GitHub Pages, the app's daily download |
| `4cdf3db` | Handoff wording: a guide, not an app store |
| `8d08edc` | Release manifest and `install-release.sh`; the release app is installed on the user's machine for testing |
| `c3ec080`, `810fb56` | App icon: the penguin at a desk (made in another session) |
| `b293a62` | Linux & You renamed to Lookbook: app ID `io.github.omegaui.Lookbook`, repository `omegaui/lookbook`, catalog at `omegaui.github.io/lookbook`, new metainfo summary and description, README, landing page (`catalog/index.html`); GitHub description and homepage updated, repository renamed |
| `ec64e0c` | The Devel build is named "Lookbook (Devel)" in launchers |
| `6e3c158` | Flat look (no isometric art; `EdgePicture`, `DesktopScene`, `IconTile`, `LayerStack`), Explore's Hall of Fame shows Hyprland's ricing contest winners (refreshed by the Catalog workflow), details pages get a Screenshots gallery |
| `f099785` | 165 gallery screenshots for all 34 items, gathered by three agents from Commons, official project media and rice READMEs; the pipeline recognises images served without a Content-Type (hypr.land) |
| `9f0d309` | "How Linux Fits Together": a question mark on the Your System banner opens a dialog that draws your setup, a full desktop (Fedora + GNOME, Mutter inside) and a compositor alone (Arch + Hyprland + pieces you add) as layers you pick; each layer explains what it is, how to change it, and links to examples (`fits_together.rs`, `FitsTogetherDialog`) |
| `d5c7ca9` | Main menu: Refresh Catalog (Ctrl+R; downloads now and reopens the window on a newer catalog, or says it is up to date), Keyboard Shortcuts (`Adw.ShortcutsDialog`, Ctrl+?; needs the libadwaita `v1_8` feature), What Is Lookbook? (F1; each page row opens its page), About. Alt+1 to Alt+6 open the pages. About credits the new screenshot sources and the Hall of Fame |

On 2026-11-07 the user asked for the release build (`d5c7ca9`) to be installed and the Devel app removed. `build-flatpak.sh` installs the Devel app again; the scripts that open pages and dialogs need it.

Not yet confirmed by clicking (the agent cannot click; the user is testing the release build):

- ~~Clicking a wallpaper opens the preview~~: confirmed working by the user on 2026-11-07.
- The Website button and spotlight arrows (crash fixed in `83158ca` with `swapped` handlers).
- Set as Wallpaper on GNOME/KDE, and Save.
- Can I Run It?: copy button, Get Boxes / Get Impression / Website links.
- View Wallpaper on the wallpaper spotlight.
- Rices: filter toggles, Get It, and Play on the clip previews (Caelestia, KooL's).
- What's New and Hall of Fame cards opening the browser, and Hall of Fame "See All".
- Screenshots gallery: arrows, swiping, the Source link.
- Main menu: Refresh Catalog, Keyboard Shortcuts, What Is Lookbook? (its page rows and both buttons), and the keyboard shortcuts themselves.
- "How Linux Fits Together": the question mark button, picking layers, the setup toggles, and example tiles opening their details pages (the dialog should close).
- Possible bug seen once: the release app opened Explore scrolled down to the spotlight instead of the top.

## Open work, in priority order

1. ~~App icon~~ done: a penguin with headphones coding at a cosy desk, with a terminal mug, on a lavender rounded tile. The user made the artwork with ChatGPT (`.agentic-coding/design/icon/source/penguin-at-desk.png`) and chose it over our drafts (the first-commit placeholder looked like existing marks; the isometric room was too detailed; then a flat penguin in a night badge). `make_icon.py` crops the tile, replaces its soft background-removal edge with a crisp rounded one, places it on GNOME's grid (112 of 128 px) and writes 8-bit PNGs at 16 to 512 px into `data/icons/hicolor/`, plus a Devel version with hazard stripes across the desk. The symbolic icon is a penguin head with headphones. Because the icon is raster artwork, `data/icons/meson.build` installs PNG sizes instead of a scalable SVG.
2. **Check the gallery licenses marked "Not stated"** in `catalog.json` (Linux Mint and Cinnamon's linuxmint.com image, Silverblue's fedoraproject.org and Fedora Magazine images, the Agama installer, two niri release-note images, the Wayfire wiki and Wayfire-Lavender images). The KDE images are recorded as CC BY 4.0 from the kde.org website repository's README, not the pages themselves. Replace any the user is not comfortable with.
3. **Review findings** (`review.md`). Done: 1.4 (app ID matches the repository), 1.6 (no "Linux" in the name), 1.10 (summary), 1.11 (description), 1.16 (README), 1.18 (GSettings description). Next, in the review's order: the three Flathub blockers (1.1 to 1.3), then 2.1, 2.5, 2.4 and 2.6.
4. **Gallery card art**: layout drawings on compositor cards, tinted capsules on distro cards. Every item now has real screenshots, so the drawn art only shows while they load.
5. **Release prep**: a release manifest with vendored crates (`build-aux/dist-vendor.sh`; the local release manifest downloads crates during the build), screenshots in metainfo, a check of brand guidelines for logos (especially Fedora) before Flathub.
6. **App CI**: a workflow that builds the Flatpak and runs the tests on each push (only the catalog is automated today).
7. Maybe: cache the release feeds for a few hours (the GitHub Atom feeds add up to about 3 MB per launch); more rices for river, labwc and Wayfire.
8. Later: upgrade to gtk4-rs 0.11, libadwaita-rs 0.9, soup3 0.9.

## Decisions

- **Rust + GTK 4 + libadwaita, Blueprint UI, Meson + Cargo, GNOME 51 runtime.**
- **Name: Lookbook** (2026-11-07). Chosen from a shortlist (Diorama, Hopscotch, Vitrine…) so the name carries no "Linux" trademark and the app ID matches the repository. No Flathub app uses the name; other "Lookbook" products are fashion and shop apps and a Rails dev tool. The `lay-` prefix of bundled icon names (`lay-logo-fedora-symbolic`) is left from the old name; it is internal and also used in the online catalog, so it stays. The wireframe and icon concept files in `.agentic-coding/design/` keep the old name as history.
- **Logos**: Simple Icons and font-logos, cleaned to symbolic SVGs, drawn white on the brand-color tile. Bazzite comes from its press kit, labwc from its repo (CC BY-SA 4.0), Wayfire traced from wf-shell's icon. COSMIC has no usable logo yet and shows letters.
- **Protocol support** comes from wayland-explorer (MIT) per compositor, with version and date. Inside Flatpak, compositors hide privileged globals (layer shell, capture, session lock), so ideas are judged against the catalog list for the detected compositor, not the globals the app sees. The compositor is detected by `XDG_CURRENT_DESKTOP`, then by "fingerprint" globals only it offers.
- **Screen capture** counts when the compositor offers a capture protocol or its portal backend provides the ScreenCast portal (so GNOME and KDE show it). Each compositor lists its usual portals in the catalog.
- **Wallpapers**: neither source has a license. The user chose to build the section now, read the sources live (never bundle or mirror), credit them, keep a takedown list (`wallpaper_blocklist`) in the catalog, and email both owners for permission. If either says no, remove its entry from the catalog. Details: `docs/research/wallpapers.md`.
- **Hosting**: GitHub Pages from this same repo for the online catalog and a small landing page (`catalog/index.html`, the repo's homepage link); no VPS needed. The repo is public (the user asked for it on 2026-11-07).
- **Catalog pipeline**: a single hand-written `catalog.json` is the source; the workflow refreshes upstream data, commits it back and publishes it. See `catalog/README.md`.
- **Screenshots** come live from Wikimedia Commons (free licenses, credited in tooltips, the About dialog and `docs/screenshot-credits.md`), never bundled. Rice previews come live from each rice's own README, credited to the author; remove an entry on request (`docs/rices.md`).
- **Can I Run It?** asks Repology for packages (HTTP/1.1, one request a second) and judges ideas with `built_into`, so a desktop's own features are never shown as gained or given up.
- **Release builds** install locally with `install-release.sh`; a Flathub submission still needs vendored crates.

## Pitfalls already hit

- **Template callbacks need `swapped`.** In Blueprint, write `clicked => $handler() swapped;` for callbacks defined on the template class, or GTK passes the button as `self` and the app aborts.
- **A Button inside a FlowBox gets the click**; `FlowBox::child-activated` never fires. Connect the button's own `clicked`.
- **No `rotateX()` in GTK CSS.** It forces an offscreen pass that clips text (only matters if tilted art ever comes back).
- **AspectFrame inside a FlowBox collapses** to almost no height; use a fixed `height-request` with `content-fit: cover`.
- **Six header tabs need about 980px**; the breakpoint at 1020sp moves them to the bottom bar. Adding a seventh tab means raising it.
- **Hyprland and restored maximize**: GTK drew the window wider than its tile. Maximize is now restored only on stacking desktops.
- **zsh**: `LINES` is a reserved variable; don't use it in shell scripts run through zsh. zsh also does not split an unquoted `$files` into words, so `sed -i … $files` gets one long name; pipe `git grep -lz` into `xargs -0` instead.
- **WebP decodes inside the app's sandbox** (glycin, GNOME 51), so hypr.land's `.webp` Hall of Fame shots load. It fails in a bare `flatpak run org.gnome.Sdk` shell, which is not a sign it fails in the app. Copyparty thumbnails still ask for JPEG (`th=j`).
- **A newer downloaded catalog wins over the bundled one** (`online.rs` compares `generated_at`). After changing the catalog's shape, bump `generated_at` in the bundled file or a local build keeps showing the old online data until the workflow republishes.
- **`git mv` and `git rm` stage at once**: a later `git commit` of other files takes them along. Commit only the paths you mean with `git commit -- <paths>`, or unstage first.
- Rust's `Variant` boxes itself as `v` in tuples; build portal parameters with `glib::Variant::tuple_from_iter`.
- **A FlowBox fits children by their natural width**, and a `Picture`'s natural width is its image's full width, so big screenshots give one card per row. Wrap the card in `NaturalWidthCap`.
- **Widgets with a layout manager (like `adw::Bin`) never call their own `measure`.** Subclass `gtk::Widget` and lay out the child yourself when you need to change measuring.
- **`gapplication action … scroll-to 1` sends an integer**; the scroll script formats the number as a double.
- **Repology resets libsoup's HTTP/2 connections** ("Peer failed to perform TLS handshake"); `http::get_bytes_over_http1` forces HTTP/1.1 (needs the soup3 `v3_4` feature).
- **Wikimedia Commons serves only standard thumbnail widths** (for example 960px); other widths return an HTML error page. Take `thumburl` from the API with `iiurlwidth=960`.
- **GTK cannot play an `https://` video file in the sandbox** (no GVfs; `gtk_gst_media_file_source_setup_cb: assertion 'stream != NULL'`). Download the bytes with libsoup and play a `gio::MemoryInputStream`; it is seekable, so looping works.
- **A wrapping label's natural width is its whole text on one line**, which widens FlowBox cards; give long wrapping labels `max-width-chars`.
- **flatpak-builder copies the whole project folder**, including another build's folder; both manifests list the build folders under `skip`.
- **LICENSE must stay the plain GPL text.** Even a two-line name and copyright header made GitHub report the license as "Other" (`NOASSERTION`). Put project notices in the README.
- **Logs**: run `flatpak run io.github.omegaui.Lookbook.Devel > log 2>&1` with `nohup … &`; piping through `grep` holds output until the app exits.

## Design

See `.agentic-coding/design/README.md`. The canvas is https://claude.ai/artifact/8q7jT54QJK1YVrPnbaYX5u and a source copy is in `.agentic-coding/design/wireframe/`.
