#!/usr/bin/env python3
"""Builds the catalog Lookbook downloads at runtime.

The hand-written catalog lives in data/resources/catalog/catalog.json; the app
bundles that file as its fallback. This script:

1. refreshes data that changes upstream (each compositor's Wayland protocols,
   from wayland-explorer, and the Hall of Fame, from the Hyprland website's
   list of ricing contest winners), never moving back to older data;
2. checks that every hosted screenshot still loads, and drops the ones that
   are gone;
3. checks that every reference in the catalog points at something real;
4. writes the published files: v1/catalog.json, v1/manifest.json and the
   landing page (from catalog/index.html).

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
import re
import string
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "data/resources/catalog/catalog.json"
SOURCES_CONFIG = ROOT / "catalog/sources.json"
LOGO_DIR = ROOT / "data/resources/icons/scalable/apps"
INDEX_TEMPLATE = Path(__file__).resolve().parent / "index.html"

# The app reads v1/catalog.json and ignores a catalog with another schema
# version. Bump both, and publish under v2/, for changes old apps cannot read.
SCHEMA_VERSION = 1
PUBLISH_DIR = f"v{SCHEMA_VERSION}"
SITE_URL = "https://omegaui.github.io/lookbook"
USER_AGENT = "Lookbook-catalog/1 (+https://github.com/omegaui/lookbook)"
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


def fetch_text(url):
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
        return response.read().decode("utf-8")


def media_status(url):
    """Returns "ok", "gone" or "unknown" for a hosted picture or clip.

    "gone" means the server answered clearly: 404/410, or a page that is not an
    image or video (Wikimedia answers bad thumbnail sizes with an HTML page).
    Timeouts and server errors are "unknown", so a bad day upstream never
    deletes data. Some servers (hypr.land) send no Content-Type; then the
    first bytes tell whether it is a picture or clip.
    """
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT, "Range": "bytes=0-15"})
    try:
        with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
            content_type = response.headers.get("Content-Type", "")
            first_bytes = response.read(16)
    except urllib.error.HTTPError as error:
        return "gone" if error.code in (404, 410) else "unknown"
    except (urllib.error.URLError, TimeoutError, OSError):
        return "unknown"
    if content_type.startswith(("image/", "video/")):
        return "ok"
    if not content_type and looks_like_media(first_bytes):
        return "ok"
    return "gone"


def looks_like_media(first_bytes):
    """True when the bytes start like a PNG, JPEG, GIF, WebP, MP4 or WebM file."""
    signatures = (b"\x89PNG", b"\xff\xd8\xff", b"GIF8", b"\x1a\x45\xdf\xa3")
    if first_bytes.startswith(signatures):
        return True
    is_webp = first_bytes[:4] == b"RIFF" and first_bytes[8:12] == b"WEBP"
    is_mp4 = first_bytes[4:8] == b"ftyp"
    return is_webp or is_mp4


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


# Step 1b: Hall of Fame


def refresh_hall_of_fame(catalog, config, report, fetch=fetch_text):
    """Updates the Hall of Fame from the Hyprland website's list of contest winners."""
    source = config["hyprland_hall_of_fame"]
    try:
        contests = parse_hall_of_fame(fetch(source["url"]), source["image_base"], source["page"])
    except Exception as error:  # noqa: BLE001 - any failure keeps the old data
        report.warn(f"Hall of Fame: could not read the Hyprland website's contests ({error})")
        return
    apply_hall_of_fame(catalog["hall_of_fame"], contests, report)


def apply_hall_of_fame(hall_of_fame, contests, report):
    current = hall_of_fame["contests"]
    if len(contests) < len(current):
        report.warn(f"Hall of Fame: the Hyprland website lists {len(contests)} contests, down from {len(current)}; kept the old list")
        return
    if contests == current:
        return
    rice_count = sum(len(contest["rices"]) for contest in contests)
    report.change(f"Hall of Fame: {len(contests)} contests, {rice_count} rices")
    hall_of_fame["contests"] = contests


def parse_hall_of_fame(source, image_base, page):
    """Reads the contests from the website's rices.ts, newest first.

    The file is a TypeScript list of contest objects, each with a name, number,
    date and a `rices: [...]` list of flat objects. Splitting at each `rices: [`
    gives one contest's header, then its rices followed by the next header.
    """
    start = source.find("export const contests")
    if start < 0:
        raise ValueError("no contests list in the file")
    parts = re.split(r"\brices:\s*\[", source[start:])

    contests = []
    header = parts[0]
    for part in parts[1:]:
        rices_text, next_header = split_at_list_end(part)
        contests.append(contest_from(header, rices_text, image_base, page))
        header = next_header

    if not contests:
        raise ValueError("no contests found")
    return sorted(contests, key=lambda contest: contest["number"], reverse=True)


def split_at_list_end(text):
    """Splits "rice objects ] } , next contest..." where the rices list ends."""
    end = re.search(r"\]\s*\}", text)
    if not end:
        raise ValueError("a rices list never ends")
    return text[: end.start()], text[end.end() :]


def contest_from(header, rices_text, image_base, page):
    rices = [rice_from(body, image_base, page) for body in re.findall(r"\{([^{}]*)\}", rices_text)]
    if not rices:
        raise ValueError("a contest has no rices")
    return {
        "name": text_field(header, "name"),
        "number": int(number_field(header, "number")),
        "date": text_field(header, "date"),
        "rices": rices,
    }


def rice_from(body, image_base, page):
    thumbnail = text_field(body, "thumbnail")
    link = text_field(body, "dotfilesLink")
    return {
        "name": text_field(body, "name"),
        "author": text_field(body, "creator"),
        "place": text_field(body, "pretitel"),
        "image": image_base + thumbnail if thumbnail.startswith("/") else thumbnail,
        # Some winners never shared their dotfiles ("#"); link to the Hall of Fame then.
        "link": link if link.startswith(("https://", "http://")) else page,
    }


def text_field(text, key):
    match = re.search(rf"\b{key}:\s*(['\"`])((?:\\.|(?!\1).)*)\1", text, re.S)
    if not match:
        raise ValueError(f"no {key} field")
    return re.sub(r"\\(.)", r"\1", match.group(2)).strip()


def number_field(text, key):
    match = re.search(rf"\b{key}:\s*(\d+)", text)
    if not match:
        raise ValueError(f"no {key} field")
    return match.group(1)


# Step 2: hosted media


def check_hosted_media(catalog, report, status_of=media_status):
    """Drops screenshots, gallery images and Hall of Fame rices that are gone.
    Rice previews only warn: the card still works."""
    screenshots = list(hosted_screenshots(catalog))
    gallery = [shot["url"] for item in catalog_items(catalog) for shot in item.get("gallery", [])]
    previews = [(rice["name"], rice["preview"]["url"]) for rice in catalog.get("rices", [])]
    famed = [rice["image"] for contest in catalog["hall_of_fame"]["contests"] for rice in contest["rices"]]
    urls = {url for _, _, url in screenshots} | set(gallery) | {url for _, url in previews} | set(famed)
    statuses = {url: status_of(url) for url in urls}

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
    drop_gone_gallery_images(catalog, statuses, report)
    drop_gone_famed_rices(catalog["hall_of_fame"], statuses, report)


def drop_gone_gallery_images(catalog, statuses, report):
    for item in catalog_items(catalog):
        for shot in item.get("gallery", []):
            if statuses[shot["url"]] == "gone":
                report.change(f"{item['name']}: dropped a gallery image, which is gone ({shot['url']})")
        if "gallery" in item:
            item["gallery"] = [shot for shot in item["gallery"] if statuses[shot["url"]] != "gone"]


def drop_gone_famed_rices(hall_of_fame, statuses, report):
    for contest in hall_of_fame["contests"]:
        kept = [rice for rice in contest["rices"] if statuses.get(rice["image"]) != "gone"]
        for rice in contest["rices"]:
            if rice not in kept:
                report.change(f"Hall of Fame: dropped {rice['name']}, whose screenshot is gone ({rice['image']})")
        contest["rices"] = kept
    hall_of_fame["contests"] = [contest for contest in hall_of_fame["contests"] if contest["rices"]]


def hosted_screenshots(catalog):
    for item in catalog_items(catalog):
        if "screenshot" in item:
            yield item, item["name"], item["screenshot"]["url"]


def catalog_items(catalog):
    """Every distro, desktop and compositor."""
    for kind in ("distros", "desktops", "compositors"):
        yield from catalog[kind]


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
    check_hall_of_fame(catalog, problems)
    check_ideas(catalog, ids, problems)
    check_rices(catalog, ids, problems)
    check_logos(catalog, problems)
    check_galleries(catalog, problems)
    check_fingerprints(catalog, problems)
    check_id_links(catalog, ids, problems)
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


def check_hall_of_fame(catalog, problems):
    contests = catalog.get("hall_of_fame", {}).get("contests", [])
    if not contests:
        problems.append("the Hall of Fame has no contests")
    for contest in contests:
        for rice in contest["rices"]:
            for key in ("image", "link"):
                if not rice[key].startswith("https://"):
                    problems.append(f"Hall of Fame rice {rice['name']}: {key} is not an https link")


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


def check_fingerprints(catalog, problems):
    """The app picks the first compositor whose fingerprint it sees, so a
    fingerprint another compositor also offers would name the wrong one."""
    for compositor in catalog["compositors"]:
        for fingerprint in compositor.get("fingerprints", []):
            for other in catalog["compositors"]:
                if other is compositor:
                    continue
                if any(glob.startswith(fingerprint) for glob in other["protocols"]["globals"]):
                    problems.append(f"{compositor['id']}: fingerprint {fingerprint} is also offered by {other['id']}")


def check_id_links(catalog, ids, problems):
    for desktop in catalog["desktops"]:
        link = desktop.get("compositor_id")
        if link and link not in ids["compositors"]:
            problems.append(f"desktop {desktop['id']} runs on unknown compositor {link}")
    for distro in catalog["distros"]:
        link = distro.get("default_desktop_id")
        if link and link not in ids["desktops"]:
            problems.append(f"distro {distro['id']} starts on unknown desktop {link}")


def check_galleries(catalog, problems):
    """Gallery images load live from their hosts, so each needs an https link,
    the page that holds its license, and a credit."""
    for item in catalog_items(catalog):
        for shot in item.get("gallery", []):
            for key in ("url", "page"):
                if not shot.get(key, "").startswith("https://"):
                    problems.append(f"{item['id']}: a gallery image has no https {key}")
            for key in ("credit", "license"):
                if not shot.get(key):
                    problems.append(f"{item['id']}: a gallery image has no {key}")


def check_logos(catalog, problems):
    """Every logo must be in the repository, so the next app release bundles it."""
    for kind in ("distros", "desktops", "compositors", "news_sources"):
        for item in catalog.get(kind, []):
            logo = item.get("logo")
            if logo and not (LOGO_DIR / f"lay-logo-{logo}-symbolic.svg").exists():
                problems.append(f"{kind}: {item['id']} uses logo {logo}, which is not in {LOGO_DIR.relative_to(ROOT)}")


# Step 4: output


def publish(catalog, out_dir, now, live=None):
    """Writes v1/catalog.json, v1/manifest.json and index.html under out_dir.

    `live` is the catalog online now. When nothing but the date would change,
    its date is kept, so the file stays byte for byte the same: apps then get
    "not modified" and About keeps the date the data really changed.
    """
    generated_at = live["generated_at"] if live and same_content(live, catalog) else now
    published = dict(catalog, generated_at=generated_at)
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


def same_content(first, second):
    """Whether two catalogs differ only in their date."""
    def without_date(catalog):
        return {key: value for key, value in catalog.items() if key != "generated_at"}

    return without_date(first) == without_date(second)


def fetch_live_catalog(fetch=fetch_json):
    """The catalog online now, or None when it cannot be read."""
    try:
        return fetch(f"{SITE_URL}/{PUBLISH_DIR}/catalog.json")
    except Exception:  # noqa: BLE001 - without it, the date simply moves on
        return None


def keep_older_versions(out_dir, fetch=fetch_json):
    """Pages replaces the whole site on each deploy. Apps from before a schema
    bump still read their own folder, so copy every older version's live
    files into the new site, or stop: publishing without them would leave
    those apps on old data for good."""
    for version in range(1, SCHEMA_VERSION):
        folder = out_dir / f"v{version}"
        folder.mkdir(parents=True, exist_ok=True)
        for name in ("catalog.json", "manifest.json"):
            url = f"{SITE_URL}/v{version}/{name}"
            try:
                data = fetch(url)
            except Exception as error:  # noqa: BLE001
                raise SystemExit(f"could not keep {url} ({error}); refusing to publish without it") from error
            (folder / name).write_text(json.dumps(data, ensure_ascii=False, separators=(",", ":")))


def index_page(now):
    """Fills in the landing page template (index.html next to this script)."""
    template = string.Template(INDEX_TEMPLATE.read_text())
    return template.substitute(publish_dir=PUBLISH_DIR, built=now)


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
        refresh_hall_of_fame(catalog, config, report)
        check_hosted_media(catalog, report)

    problems = validate(catalog)
    if problems:
        print("The catalog has problems:", *problems, sep="\n  ")
        return 1

    now = utc_now()
    if args.write_source and report.changes:
        save_source(catalog, now)
    live = None if args.offline else fetch_live_catalog()
    publish(catalog, args.out, now, live)
    if not args.offline:
        keep_older_versions(args.out)
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
