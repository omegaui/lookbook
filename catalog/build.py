#!/usr/bin/env python3
"""Builds the catalog Linux & You downloads at runtime.

The hand-written catalog lives in data/resources/catalog/catalog.json; the app
bundles that file as its fallback. This script:

1. refreshes data that changes upstream (each compositor's Wayland protocols,
   from wayland-explorer), never moving a compositor back to older data;
2. checks that every hosted screenshot still loads, and drops the ones that
   are gone;
3. checks that every reference in the catalog points at something real;
4. writes the published files: v1/catalog.json, v1/manifest.json and a small
   index page.

It uses only the Python standard library, so it runs anywhere.

    python3 catalog/build.py --out _site                 # build
    python3 catalog/build.py --out _site --write-source  # also save refreshed data
    python3 catalog/build.py --out _site --offline       # no network: check and publish
"""

import argparse
import datetime
import hashlib
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "data/resources/catalog/catalog.json"
SOURCES_CONFIG = ROOT / "catalog/sources.json"
LOGO_DIR = ROOT / "data/resources/icons/scalable/apps"

# The app reads v1/catalog.json and ignores a catalog with another schema
# version. Bump both, and publish under v2/, for changes old apps cannot read.
SCHEMA_VERSION = 1
PUBLISH_DIR = "v1"
USER_AGENT = "LinuxAndYou-catalog/1 (+https://github.com/omegaui/linux_and_you)"
TIMEOUT_SECONDS = 30
# If more media than this fails, the network is more likely at fault than the
# images: stop, so the last good catalog stays published.
MAX_MISSING_MEDIA_SHARE = 0.25
# A protocol refresh that loses more than half of a compositor's globals is
# more likely a broken dump than a real change.
MIN_KEPT_GLOBALS_SHARE = 0.5


class Report:
    """What changed and what looked wrong, printed and added to the job summary."""

    def __init__(self):
        self.changes = []
        self.warnings = []

    def change(self, text):
        self.changes.append(text)
        print(f"changed: {text}")

    def warn(self, text):
        self.warnings.append(text)
        print(f"warning: {text}")

    def summary(self):
        lines = ["## Catalog build", ""]
        lines += [f"- {change}" for change in self.changes] or ["- No data changes."]
        if self.warnings:
            lines += ["", "### Warnings", ""] + [f"- {warning}" for warning in self.warnings]
        return "\n".join(lines) + "\n"


# Network


def fetch_json(url):
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
        return json.load(response)


def media_status(url):
    """Returns "ok", "gone" or "unknown" for a hosted picture or clip.

    "gone" means the server answered clearly: 404/410, or a page that is not an
    image or video (Wikimedia answers bad thumbnail sizes with an HTML page).
    Timeouts and server errors are "unknown", so a bad day upstream never
    deletes data.
    """
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT, "Range": "bytes=0-0"})
    try:
        with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
            content_type = response.headers.get("Content-Type", "")
    except urllib.error.HTTPError as error:
        return "gone" if error.code in (404, 410) else "unknown"
    except (urllib.error.URLError, TimeoutError, OSError):
        return "unknown"
    return "ok" if content_type.startswith(("image/", "video/")) else "gone"


# Step 1: protocols


def refresh_protocols(catalog, config, report, fetch=fetch_json):
    """Updates each compositor's protocol list when wayland-explorer has newer data."""
    source = config["wayland_explorer"]
    for compositor in catalog["compositors"]:
        upstream_id = source["compositors"].get(compositor["id"])
        if not upstream_id:
            continue
        try:
            dump = fetch(source["url"].format(id=upstream_id))
        except Exception as error:  # noqa: BLE001 - any failure keeps the old data
            report.warn(f"{compositor['name']}: could not read wayland-explorer ({error})")
            continue
        apply_protocol_dump(compositor, dump, report)


def apply_protocol_dump(compositor, dump, report):
    current = compositor["protocols"]
    checked_on = day_from_milliseconds(dump["generationTimestamp"])
    if checked_on <= current["checked_on"]:
        return

    globals_ = sorted({entry["interface"] for entry in dump["globals"]})
    if len(globals_) < len(current["globals"]) * MIN_KEPT_GLOBALS_SHARE:
        report.warn(
            f"{compositor['name']}: wayland-explorer lists {len(globals_)} globals, "
            f"down from {len(current['globals'])}; kept the old list"
        )
        return

    report.change(
        f"{compositor['name']} protocols: {current['version']} ({current['checked_on']}) "
        f"to {dump['version']} ({checked_on}), {len(globals_)} globals"
    )
    compositor["protocols"] = {"version": dump["version"], "checked_on": checked_on, "globals": globals_}


def day_from_milliseconds(milliseconds):
    moment = datetime.datetime.fromtimestamp(milliseconds / 1000, tz=datetime.timezone.utc)
    return moment.date().isoformat()


# Step 2: hosted media


def check_hosted_media(catalog, report, status_of=media_status):
    """Drops screenshots that are gone. Rice previews only warn: the card still works."""
    screenshots = list(hosted_screenshots(catalog))
    previews = [(rice["name"], rice["preview"]["url"]) for rice in catalog.get("rices", [])]
    statuses = {url: status_of(url) for url in {url for _, _, url in screenshots} | {url for _, url in previews}}

    gone = [url for url, status in statuses.items() if status == "gone"]
    if statuses and len(gone) / len(statuses) > MAX_MISSING_MEDIA_SHARE:
        raise SystemExit(f"{len(gone)} of {len(statuses)} media links look gone; refusing to publish")

    for item, name, url in screenshots:
        if statuses[url] == "gone":
            del item["screenshot"]
            report.change(f"{name}: dropped its screenshot, which is gone ({url})")
    for name, url in previews:
        if statuses[url] == "gone":
            report.warn(f"rice {name}: preview is gone ({url})")


def hosted_screenshots(catalog):
    for kind in ("distros", "desktops", "compositors"):
        for item in catalog[kind]:
            if "screenshot" in item:
                yield item, item["name"], item["screenshot"]["url"]


# Step 3: checks


def validate(catalog):
    """Returns a list of problems. An empty list means the catalog is fine to publish."""
    problems = []
    if catalog.get("schema_version") != SCHEMA_VERSION:
        problems.append(f"schema_version is {catalog.get('schema_version')}, expected {SCHEMA_VERSION}")

    ids = {kind: unique_ids(catalog, kind, problems) for kind in ("distros", "desktops", "compositors")}
    unique_ids(catalog, "ideas", problems)
    unique_ids(catalog, "rices", problems)
    unique_ids(catalog, "news_sources", problems)
    wallpaper_sources = unique_ids(catalog, "wallpaper_sources", problems)

    check_spotlights(catalog, ids, wallpaper_sources, problems)
    check_collections(catalog, ids, problems)
    check_hall_of_fame(catalog, ids, problems)
    check_ideas(catalog, ids, problems)
    check_rices(catalog, ids, problems)
    check_logos(catalog, problems)
    return problems


def unique_ids(catalog, kind, problems):
    seen = set()
    for item in catalog.get(kind, []):
        if item["id"] in seen:
            problems.append(f"{kind}: {item['id']} appears twice")
        seen.add(item["id"])
    return seen


def plural(kind):
    return {"distro": "distros", "desktop": "desktops", "compositor": "compositors"}.get(kind)


def check_spotlights(catalog, ids, wallpaper_sources, problems):
    for spotlight in catalog["spotlights"]:
        if spotlight["id"] not in ids.get(plural(spotlight["kind"]), set()):
            problems.append(f"spotlight points at unknown {spotlight['kind']} {spotlight['id']}")
    wallpaper = catalog.get("wallpaper_spotlight")
    if wallpaper and wallpaper["source_id"] not in wallpaper_sources:
        problems.append(f"wallpaper spotlight points at unknown source {wallpaper['source_id']}")


def check_collections(catalog, ids, problems):
    for collection in catalog["collections"]:
        known = ids.get(plural(collection["kind"]), set())
        for member in collection["members"]:
            if member not in known:
                problems.append(f"collection {collection['title']} lists unknown {collection['kind']} {member}")


def check_hall_of_fame(catalog, ids, problems):
    for screenshot in catalog["screenshots"]:
        if screenshot["subject"] not in ids["desktops"] | ids["compositors"]:
            problems.append(f"Hall of Fame entry {screenshot['title']} shows unknown {screenshot['subject']}")


def check_ideas(catalog, ids, problems):
    sessions = ids["desktops"] | ids["compositors"]
    for idea in catalog["ideas"]:
        for item in idea.get("built_into", []):
            if item not in sessions:
                problems.append(f"idea {idea['id']} is built into unknown {item}")
        for route in idea["routes"]:
            for desktop in route.get("desktops", []):
                if desktop not in ids["desktops"]:
                    problems.append(f"idea {idea['id']} has a route for unknown desktop {desktop}")


def check_rices(catalog, ids, problems):
    for rice in catalog.get("rices", []):
        if not rice["compositors"]:
            problems.append(f"rice {rice['id']} runs on no compositor")
        for compositor in rice["compositors"]:
            if compositor not in ids["compositors"]:
                problems.append(f"rice {rice['id']} runs on unknown compositor {compositor}")


def check_logos(catalog, problems):
    """Every logo must be in the repository, so the next app release bundles it."""
    for kind in ("distros", "desktops", "compositors", "news_sources"):
        for item in catalog.get(kind, []):
            logo = item.get("logo")
            if logo and not (LOGO_DIR / f"lay-logo-{logo}-symbolic.svg").exists():
                problems.append(f"{kind}: {item['id']} uses logo {logo}, which is not in {LOGO_DIR.relative_to(ROOT)}")


# Step 4: output


def publish(catalog, out_dir, now):
    """Writes v1/catalog.json, v1/manifest.json and index.html under out_dir."""
    published = dict(catalog, generated_at=now)
    body = json.dumps(published, ensure_ascii=False, separators=(",", ":")).encode()

    version_dir = out_dir / PUBLISH_DIR
    version_dir.mkdir(parents=True, exist_ok=True)
    (version_dir / "catalog.json").write_bytes(body)

    manifest = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": now,
        "sha256": hashlib.sha256(body).hexdigest(),
        "bytes": len(body),
    }
    (version_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    (out_dir / "index.html").write_text(index_page(now))


def index_page(now):
    return f"""<!doctype html>
<html lang="en">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Linux &amp; You catalog</title>
<style>body{{font:16px/1.5 system-ui,sans-serif;max-width:40rem;margin:3rem auto;padding:0 1rem}}</style>
<h1>Linux &amp; You catalog</h1>
<p>The data the <a href="https://github.com/omegaui/linux_and_you">Linux &amp; You</a> app reads: distros, desktops, compositors, rices and where to find their latest releases.</p>
<ul>
<li><a href="{PUBLISH_DIR}/catalog.json">{PUBLISH_DIR}/catalog.json</a></li>
<li><a href="{PUBLISH_DIR}/manifest.json">{PUBLISH_DIR}/manifest.json</a></li>
</ul>
<p>Built {now}.</p>
</html>
"""


def save_source(catalog, now):
    """Writes refreshed data back to the hand-written file, in its usual format."""
    saved = dict(catalog, generated_at=now)
    SOURCE.write_text(json.dumps(saved, indent=2, ensure_ascii=False) + "\n")


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


# Main


def parse_args():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--out", type=Path, required=True, help="folder to write the published site into")
    parser.add_argument("--write-source", action="store_true", help="save refreshed data back to the catalog file")
    parser.add_argument("--offline", action="store_true", help="skip every network step")
    return parser.parse_args()


def main():
    args = parse_args()
    catalog = json.loads(SOURCE.read_text())
    config = json.loads(SOURCES_CONFIG.read_text())
    report = Report()

    if not args.offline:
        refresh_protocols(catalog, config, report)
        check_hosted_media(catalog, report)

    problems = validate(catalog)
    if problems:
        print("The catalog has problems:", *problems, sep="\n  ")
        return 1

    now = utc_now()
    if args.write_source and report.changes:
        save_source(catalog, now)
    publish(catalog, args.out, now)
    write_job_summary(report)
    print(f"Published {args.out / PUBLISH_DIR / 'catalog.json'}")
    return 0


def write_job_summary(report):
    summary_file = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_file:
        with open(summary_file, "a", encoding="utf-8") as summary:
            summary.write(report.summary())


if __name__ == "__main__":
    sys.exit(main())
