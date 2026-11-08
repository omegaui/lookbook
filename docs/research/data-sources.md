# Live data sources

Research date: 2026-11-07. Every endpoint below was fetched with `curl` on that date unless it says **unverified**.

## Goal

Syspedia shows current information about distros, desktops and compositors without shipping a new app release. App releases are for app features only.

The app does not call third-party APIs. A separate data repository collects data on a schedule, checks it, and publishes one catalog. The app downloads that catalog, caches it, and falls back to a copy bundled with the app.

## Distros

| Source | Endpoint | Gives | License | Use |
| --- | --- | --- | --- | --- |
| endoflife.date | `https://endoflife.date/api/v1/products/{id}`, `/api/v1/tags/linux-distribution`, `/api/v1/products/full` | Release cycles, codenames, release and end-of-life dates, latest point release | MIT | High. No Arch or most rolling distros, no GNOME, no unreleased versions |
| distro-info-data | `https://salsa.debian.org/debian/distro-info-data/-/raw/main/ubuntu.csv`, `debian.csv` | Versions, codenames, release and EOL dates, **including future releases** (Ubuntu 27.04, Debian 14) | ISC | High for upcoming Debian and Ubuntu releases |
| Fedora Bodhi | `https://bodhi.fedoraproject.org/releases/?exclude_archived=true` | Current and pending Fedora releases, EOL dates | Not checked | High |
| Fedora schedule | `https://fedorapeople.org/groups/schedule/f-46/f-46-key.ics` | Upcoming Beta and GA dates | Not checked | High for upcoming releases |
| Fedora downloads | `https://fedoraproject.org/releases.json` | Editions, spins, atomic variants | Not checked | Medium |
| Wikidata SPARQL | `https://query.wikidata.org/sparql` (instances of `wd:Q131669`) | 823 distros; logo (P154), website (P856), based on (P144), package manager (P3033), versions with dates (P348 + P577) | CC0 | High for metadata, medium for versions |
| Wikimedia Commons | `https://commons.wikimedia.org/w/api.php?action=query&prop=imageinfo&iiprop=url\|extmetadata&iiurlwidth=128&titles=File:X.svg` | Logo thumbnails plus the license of each file | Per file | High, but check each file's license |
| distrobox | `https://raw.githubusercontent.com/89luca89/distrobox/main/docs/compatibility.md` | Container images per distro and version | GPL-3.0 | Medium. Hand-edited Markdown; lags behind releases. Useful for "try it in a container" |
| simple-icons | `https://cdn.jsdelivr.net/npm/simple-icons@16.36.0/data/simple-icons.json`, `/icons/{slug}.svg` | Monochrome SVG logos with brand colors for most distros, GNOME, KDE Plasma, Xfce, Cinnamon, Hyprland, sway, niri | Project CC0, but some icons have their own terms (Debian CC-BY-SA-3.0, Fedora custom) | High; symbolic style fits libadwaita |
| font-logos | `https://github.com/Lukas-W/font-logos` | 134 distro and desktop logo SVGs | Unlicense; logos remain trademarks | Medium |
| DistroWatch | `https://distrowatch.com/news/dwd.xml` (RSS) | Latest releases | No API; robots.txt blocks AI crawlers; terms unverified | Low. Do not scrape; at most link out |

Missing from simple-icons: Budgie, COSMIC, Bazzite, SteamOS, Mageia, Vanilla OS, postmarketOS, Nobara.

### The user's own distro

Inside the Flatpak sandbox the host file is at `/run/host/os-release`. Its `LOGO=` field names an icon from the icon theme. The host's icons are at `/run/host/share/icons`; add that path to the GTK icon theme search path. Tested in a GNOME 50 sandbox on CachyOS: `ID=cachyos`, `LOGO=cachyos`.

## Desktops and compositors

| Source | Endpoint | Gives | Use |
| --- | --- | --- | --- |
| GNOME versions | `https://download.gnome.org/sources/gnome-shell/cache.json` | All GNOME Shell versions (latest 51.1 on 2026-11-07) | High. Do not use `download.gnome.org/core/`, which stops at 47 |
| GNOME schedule | `https://release.gnome.org/calendar/schedule.ics` | Upcoming GNOME releases (52.alpha 2027-01-09) | High |
| KDE Plasma | endoflife.date `kde-plasma`; `https://invent.kde.org/api/v4/projects/plasma%2Fplasma-workspace/repository/tags` | Plasma releases (6.8.2) | High |
| GitHub releases | `https://api.github.com/repos/{owner}/{repo}/releases/latest` or `https://github.com/{owner}/{repo}/releases.atom` | Latest releases | High. 60 requests/hour without a token, 1,000/hour per repo with `GITHUB_TOKEN` in Actions |
| Codeberg | `https://codeberg.org/api/v1/repos/river/river/releases` | river releases | High |
| GNOME GitLab | `https://gitlab.gnome.org/api/v4/projects/GNOME%2Fmutter/repository/tags` | Mutter tags | Medium |
| Repology | `https://repology.org/api/v1/project/{name}` | Which distros ship a package, and which version | Medium. At most 1 request/second, a User-Agent with a contact link, and the API may change. Use from CI only |

Things to handle per project:

- niri moved to `niri-wm/niri`.
- cosmic-comp has no releases; use `pop-os/cosmic-epoch` (epoch-1.11.0 on 2026-10-20).
- Cinnamon's "latest release" on GitHub is `master.mint27`; read tags instead.

### Wayland protocol support per compositor

[wayland-explorer](https://github.com/vially/wayland-explorer) (MIT) is the best source.

- Data: `src/data/compositors/{id}.json`, one file per compositor (20 files: cage, cosmic, gamescope, hyprland, jay, kwin, labwc, louvre, mangowm, mir, muffin, mutter, niri, phoc, qtile, river, sway, treeland, wayfire, weston).
- Format: `{ generationTimestamp, version, globals: [{ interface, version }] }`, produced by `wlprobe` on a running compositor.
- Display names and protocol metadata live in TypeScript files (`compositor-registry.ts`, `protocol-registry.ts`). To map an interface to its protocol, parse the protocol XML files from its submodules in CI.
- Some entries are about a year old (Hyprland 0.52.1, sway 1.11, Wayfire 0.9.0). Mutter 51, KWin 6.7 and river are recent.
- The data shows what one normal client saw. Privileged or config-dependent globals can be missing.

Plan: read it as an input, run `wlprobe` ourselves for stale entries, and send fresh dumps upstream.

## Ecosystem: extensions and plugins

| Source | Endpoint | Gives | Use |
| --- | --- | --- | --- |
| extensions.gnome.org (new API) | `https://extensions.gnome.org/api/v1/extensions/?ordering=-downloads\|-popularity\|-updated&shell_version=51&page_size=…`; schema at `/api/schema/` | 5,940 extensions with names, descriptions, image URLs, ratings, supported Shell versions | High. Rate limits and terms unverified |
| extensions.gnome.org (old API) | `https://extensions.gnome.org/extension-query/?sort=popularity\|downloads&shell_version=51` | Same data with download counts | High |
| KDE Store (OCS) | `https://api.kde-look.org/ocs/v1/content/data?categories=705x706&sortmode=down&format=json` | Plasma 6 extensions (705), applets (706), KWin effects (719), KWin scripts (720), with previews and scores | Medium to high. Terms unverified |
| GitHub topic search | `https://api.github.com/search/repositories?q=topic:hyprland&sort=stars` | Trending repositories by topic (hyprland, gnome-shell-extension, niri, kwin-script, plasma-widget) | Medium. 10 searches/minute without a token; top results are often dotfiles, so filter |

## Hosting

| Option | Facts | Role |
| --- | --- | --- |
| GitHub Pages | 1 GB site, soft limit 100 GB/month bandwidth, served with `max-age=600`, ETag and CORS. No commercial use (fine for a free app) | Main source |
| jsDelivr | No bandwidth limit; files over 20 MB not served; exact tags are cached permanently | Second source, pinned to a tag |
| raw.githubusercontent.com | `max-age=300`, no service promise; GitHub's policy forbids excessive bandwidth | Last fallback only |

GitHub Actions schedules run at most every 5 minutes and can be delayed or dropped. In a public repo, scheduled workflows are disabled after 60 days with no repository activity. A daily data commit keeps them active.

Flathub has no rule against `--share=network`, but permissions must be minimal. The build has no network access, so the bundled catalog snapshot must be a source in the Flatpak manifest.

## Recommended architecture

1. **Data repository** (`syspedia-data`):
   - One adapter per source.
   - One hand-written YAML file per distro, desktop and compositor, for what no API gives: our own short descriptions, release model, default desktop, and which release source to read.
   - An `icons/` folder of checked SVGs, each with its license and source recorded.
2. **Scheduled workflow**, daily at an odd minute (for example 04:17 UTC) plus a manual trigger:
   - Daily: endoflife.date, distro-info-data, Bodhi and the Fedora schedule, GNOME versions and schedule, KDE, release feeds.
   - Weekly: Wikidata, wayland-explorer and protocol XML, extension stores, distrobox, Repology.
3. **Checks before publishing**:
   - Validate against a JSON Schema.
   - Keep the last good catalog if a source fails, a version goes backwards, or too many items disappear.
   - Record `fetched_at` per source, and open an issue when data gets stale.
4. **Output**:
   - `v1/catalog.json` with `schema_version` and `generated_at`, split into a core file and an ecosystem file.
   - Icons named by content hash.
   - Published to GitHub Pages and tagged `data-YYYYMMDD` for jsDelivr.
5. **App**:
   - Fetch at most once a day with `If-None-Match`, and cache in the sandbox's cache folder.
   - Fall back in order: GitHub Pages, then jsDelivr, then the cached copy, then the bundled snapshot.
   - Ignore a catalog with an unknown major schema version.
   - Show the "data as of" date.
   - Load extension screenshots from the stores' own URLs instead of mirroring them.

A VPS is not needed for this. It would only be needed for features a static file cannot provide, such as users submitting their own screenshots.

## Risks

1. **Logos are trademarks**, even when the file is public domain. Use them only to identify each project, follow brand guidelines, keep them out of the app's own icon, and ship an attributions screen. Check whether "Linux" in the app name needs permission from the Linux Foundation before submitting to Flathub (**unverified**).
2. **DistroWatch** has no API and blocks crawlers. Do not scrape it or copy its descriptions.
3. **Upstream data drifts**: projects move, release conventions differ, endoflife.date lacks rolling distros and future versions, the Repology API is unstable, and some wayland-explorer entries are old.
4. **Data licenses are mixed** (CC0, MIT, ISC, GPL-3.0). Ship a NOTICE file listing them.
5. **Extension store content is user-submitted.** Filter by downloads and rating, and link to the store instead of re-hosting.
6. **Scheduled workflows can be delayed, dropped or disabled.** The app shows the data date, and the data repository alerts when data is stale.
