# The catalog

Linux & You's data — distros, desktops, compositors, rices, ideas, release
feeds — lives in one hand-written file, `data/resources/catalog/catalog.json`.
The app bundles that file, and also downloads the copy published at:

**https://omegaui.github.io/linux_and_you/v1/catalog.json**

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
   - checks every hosted screenshot and rice preview, drops screenshots that
     are clearly gone, and stops if more than a quarter look gone (the network
     is more likely at fault);
   - checks that every reference points at something real and every logo file
     exists;
   - writes `v1/catalog.json`, `v1/manifest.json` (hash, size, date) and an
     index page;
3. commits refreshed data back to `main` as "Refresh catalog data", so the
   snapshot in the next app release is current too;
4. deploys to GitHub Pages.

If any step fails, nothing is deployed and the last good catalog stays online.
Pull requests that touch the catalog run steps 1 and 2 only.

## In the app

`src/catalog/online.rs` checks for a new copy at most once a day, with
`If-None-Match`, and keeps it in the app's cache folder
(`~/.var/app/io.github.omegaui.LinuxAndYou/cache/linux-and-you/`). The next
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
`src/catalog/online.rs`, and publishing under `v2/`. Older apps then keep
reading `v1/`, so leave the last `v1` build in place.

Logos in the catalog must exist in `data/resources/icons/scalable/apps/`. An
app release that does not bundle a new logo shows the item's letters instead.

## Keeping the schedule alive

GitHub turns off scheduled workflows in public repositories after 60 days
without activity. Data refresh commits count as activity, but if upstream data
stays the same for two months, re-enable the workflow from the Actions tab.
