# The catalog

Lookbook's data — distros, desktops, compositors, rices, ideas, release
feeds — lives in one hand-written file, `data/resources/catalog/catalog.json`.
The app bundles that file, and also downloads the copy published at:

**https://omegaui.github.io/lookbook/v1/catalog.json**

so data can change without an app release.

## How it updates

The **Catalog** workflow (`.github/workflows/catalog.yml`) runs every day at
04:17 UTC, on every push to `main` that touches the catalog, and by hand from
the Actions tab. It:

1. tests the builder (`catalog/test_build.py`);
2. runs `catalog/build.py`, which
   - refreshes each compositor's Wayland protocol list from
     [wayland-explorer](https://github.com/vially/wayland-explorer) (MIT) when
     it has newer data, never moving a compositor back to older data, and
     ignoring a dump that loses more than half the protocols;
   - refreshes the Hall of Fame from the Hyprland website's list of ricing
     contest winners (`src/routes/hall_of_fame/rices.ts` in
     [hyprland-website](https://github.com/hyprwm/hyprland-website),
     BSD-3-Clause), keeping the old list if the new one has fewer contests or
     cannot be read. The screenshots stay on hypr.land;
   - checks every hosted screenshot, gallery image, Hall of Fame screenshot and
     rice preview, drops screenshots and Hall of Fame entries that are clearly
     gone, and stops if more than a quarter look gone (the network is more
     likely at fault);
   - checks that every reference points at something real (including
     desktop-to-compositor and distro-to-desktop ids), that no compositor's
     fingerprint is offered by another one, and that every logo file exists;
   - writes `v1/catalog.json`, `v1/manifest.json` (hash, size, date, for
     people and tools checking the site; the app reads only the catalog) and
     the landing page from `catalog/index.html`. When nothing but the date
     changed, it keeps the live catalog's date, so the file stays the same and
     apps get "not modified";
   - after a schema bump, copies every older version's live files into the
     new site, so apps that still read them keep working;
2b. parses the published catalog with the app's own model
   (`catalog/model-check`), so a value the app would reject is never
   published;
3. commits refreshed data back to `main` as "Refresh catalog data", so the
   snapshot in the next app release is current too;
4. deploys to GitHub Pages.

If any step fails, nothing is deployed and the last good catalog stays online.
Pull requests that touch the catalog run steps 1 and 2 only.

## In the app

`src/catalog/online.rs` checks for a new copy at most once a day, with
`If-None-Match`, and keeps it in the app's cache folder
(`~/.var/app/io.github.omegaui.Lookbook/cache/lookbook/`). The next
launch uses it if it parses, has the schema version the app reads, and is newer
than the bundled snapshot; otherwise the app uses the bundled snapshot. The
About dialog shows the date of the catalog in use.

## Editing

Edit `data/resources/catalog/catalog.json`, then check it locally:

```sh
python3 -m unittest discover -s catalog -p 'test_*.py'
python3 catalog/build.py --out /tmp/site --offline   # add network checks by leaving out --offline
.agentic-coding/scripts/cargo-sdk.sh test            # the app's own catalog tests
```

New fields must be optional (`#[serde(default)]` or `Option`) so apps already
installed can still read the catalog. Anything an older app cannot read — a
removed or renamed field, a new value in an enum such as a compositor `kind` —
needs `SCHEMA_VERSION` raised in both `catalog/build.py` and
`src/catalog/online.rs`; both build the folder name (`v2/`) from it. Older
apps keep reading `v1/`, which `build.py` copies from the live site into
every later build.

Logos in the catalog must exist in `data/resources/icons/scalable/apps/`. An
app release that does not bundle a new logo shows the item's letters instead.

## Keeping the schedule alive

GitHub turns off scheduled workflows in public repositories after 60 days
without activity. Data refresh commits count as activity, but if upstream data
stays the same for two months, re-enable the workflow from the Actions tab.
