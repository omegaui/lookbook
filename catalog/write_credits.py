#!/usr/bin/env python3
"""Writes docs/screenshot-credits.md from the bundled catalog.

    python3 catalog/write_credits.py

The app shows the same credits in each picture's tooltip and in the gallery
captions; this page lists them all in one place. The online catalog can move
ahead of the bundled one, so run this again before a release.
"""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "data/resources/catalog/catalog.json"
OUTPUT = ROOT / "docs/screenshot-credits.md"
KINDS = (("distros", "Distros"), ("desktops", "Desktops"), ("compositors", "Compositors"))

INTRO = """# Screenshot credits

Syspedia never bundles screenshots. Each one loads live from where it is
hosted: most from [Wikimedia Commons](https://commons.wikimedia.org), the
rest from each project's own website or repository, or from a rice's README.
In the app, each screenshot's tooltip or caption names its author and
license, and links to its source page.

This page lists the screenshots in the catalog bundled with this version
(generated {date}). The catalog online may have changed since.
"""

HALL_OF_FAME = """## Hall of Fame

The Hall of Fame shows the winners of
[Hyprland's ricing contests]({page}), read from the Hyprland website. Each
screenshot is hosted on hypr.land and belongs to the rice's author.

| Contest | Place | Rice | Author |
| --- | --- | --- | --- |
"""


def main():
    catalog = json.loads(CATALOG.read_text())
    lines = [INTRO.format(date=catalog["generated_at"][:10])]
    for key, title in KINDS:
        lines.append(f"## {title}\n")
        for item in catalog[key]:
            lines += item_section(item)
    lines.append(hall_of_fame(catalog["hall_of_fame"]))
    OUTPUT.write_text("\n".join(lines).rstrip("\n") + "\n")
    print(f"Wrote {OUTPUT.relative_to(ROOT)}")


def item_section(item):
    shots = item.get("gallery") or ([item["screenshot"]] if "screenshot" in item else [])
    if not shots:
        return []
    rows = [f"### {item['name']}\n", "| Shows | Source | Author | License |", "| --- | --- | --- | --- |"]
    for shot in shots:
        caption = shot.get("caption", "Screenshot")
        rows.append(f"| {cell(caption)} | [page]({shot['page']}) | {cell(shot['credit'])} | {cell(shot['license'])} |")
    rows.append("")
    return rows


def hall_of_fame(hall):
    text = HALL_OF_FAME.format(page=hall["page"])
    for contest in hall["contests"]:
        for rice in contest["rices"]:
            text += f"| {cell(contest['name'])}, {cell(contest['date'])} | {cell(rice['place'])} | [{cell(rice['name'])}]({rice['link']}) | {cell(rice['author'])} |\n"
    return text


def cell(text):
    """Table cells cannot hold a pipe or a line break."""
    return str(text).replace("|", "\\|").replace("\n", " ")


if __name__ == "__main__":
    main()
