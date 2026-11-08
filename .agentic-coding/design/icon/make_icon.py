#!/usr/bin/env python3
"""Builds the Syspedia app icons from the chosen artwork: a penguin with
headphones coding at a cosy desk, with a terminal mug (source/penguin-at-desk.png,
made by the user with ChatGPT).

    python3 .agentic-coding/design/icon/make_icon.py

The source is a 1254 px PNG whose rounded tile has a soft, glowing edge left by
background removal. This script:

1. crops the tile and makes it exactly square;
2. gives it a crisp rounded edge, a little inside the soft one, with the tile
   fully opaque inside;
3. places it on GNOME's 128 grid (the tile fills 112 of 128 px);
4. writes PNGs at every size GNOME looks for, plus a development version with
   GNOME's hazard stripes across the desk, and a one-color symbolic SVG.

Needs ImageMagick (`magick`) and rsvg-convert (for the preview).
"""

import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent.parent
SOURCE = HERE / "source/penguin-at-desk.png"
WORK = HERE / "render"
ICONS = ROOT / "data/icons"
APP_ID = "io.github.omegaui.Syspedia"
FONT = "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf"

# Where the tile sits in the source image (measured where it is at least 95% opaque).
TILE_CROP = "897x906+174+185"
# The tile's corner radius in the source is about 220 of its 897 px.
CORNER_SHARE = 220 / 897
# How far inside the soft edge the clean edge goes, at 1024 px.
EDGE_INSET = 12
MASTER = 1024
# The tile fills 112 of the 128 px grid.
TILE_SHARE = 112 / 128
# The desk band, where the development stripes go, starts this far down the tile.
DESK_TOP_SHARE = 0.87
SIZES = [16, 24, 32, 48, 64, 128, 256, 512]

STRIPE_YELLOW = "#f6d32d"
STRIPE_DARK = "#241f31"


def magick(*args):
    subprocess.run(["magick", *map(str, args)], check=True)


def clean_tile():
    """The tile, square, with a crisp rounded edge and no glow."""
    tile = WORK / "tile.png"
    mask = WORK / "mask.png"
    magick(SOURCE, "-crop", TILE_CROP, "+repage", "-resize", f"{MASTER}x{MASTER}!", tile)

    radius = round(MASTER * CORNER_SHARE) - EDGE_INSET
    last = MASTER - 1 - EDGE_INSET
    magick(
        "-size", f"{MASTER}x{MASTER}", "xc:black", "-fill", "white",
        "-draw", f"roundrectangle {EDGE_INSET},{EDGE_INSET} {last},{last} {radius},{radius}", mask,
    )
    clean = WORK / "tile-clean.png"
    # Fully opaque tile, then keep only what lies inside the mask.
    magick(tile, "-alpha", "off", "-alpha", "set", "(", mask, "-alpha", "copy", ")", "-compose", "DstIn", "-composite", clean)
    return clean


def with_devel_stripes(tile):
    """The tile with yellow and dark diagonal stripes across the desk."""
    stripes = WORK / "stripes.png"
    lines = []
    for offset in range(-MASTER, 2 * MASTER, 72):
        lines += ["-draw", f"line {offset},{MASTER} {offset + MASTER},0"]
    magick("-size", f"{MASTER}x{MASTER}", f"xc:{STRIPE_YELLOW}", "-stroke", STRIPE_DARK, "-strokewidth", 36, *lines, stripes)

    desk_top = round(MASTER * DESK_TOP_SHARE)
    band = WORK / "band.png"
    magick(stripes, "-crop", f"{MASTER}x{MASTER - desk_top}+0+{desk_top}", "+repage", band)
    devel = WORK / "tile-devel.png"
    magick(tile, band, "-geometry", f"+0+{desk_top}", "-compose", "SrcAtop", "-composite", devel)
    return devel


def on_grid(tile, name):
    """The tile centred on a transparent square, filling 112 of 128 px."""
    tile_size = round(MASTER * TILE_SHARE)
    master = WORK / f"{name}-master.png"
    magick(
        "-size", f"{MASTER}x{MASTER}", "xc:none",
        "(", tile, "-resize", f"{tile_size}x{tile_size}", ")",
        "-gravity", "center", "-composite", master,
    )
    return master


def export_sizes(master, icon_name):
    """PNGs in data/icons/hicolor/<size>x<size>/apps/, sharpened a little when small."""
    for size in SIZES:
        folder = ICONS / "hicolor" / f"{size}x{size}" / "apps"
        folder.mkdir(parents=True, exist_ok=True)
        sharpen = ["-unsharp", "0x0.6+0.6+0"] if size <= 48 else []
        magick(
            master, "-filter", "Lanczos", "-resize", f"{size}x{size}", *sharpen, "-strip",
            # 8 bits per channel: the source is 8-bit, and 16-bit PNGs are four times larger.
            "-depth", "8", "-define", "png:compression-level=9", "-define", "png:compression-filter=5",
            folder / f"{icon_name}.png",
        )


def symbolic_icon():
    """A penguin's head with headphones, in one color."""
    return """<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
  <path fill="#2e3436" fill-rule="evenodd" d="M 8,3 C 11.6,3 13.5,5.8 13.5,9.2 C 13.5,12.6 11.2,15 8,15 C 4.8,15 2.5,12.6 2.5,9.2 C 2.5,5.8 4.4,3 8,3 Z M 6,7.4 C 4.6,7.4 4,8.6 4,10 C 4,12 5.6,13.4 8,13.4 C 10.4,13.4 12,12 12,10 C 12,8.6 11.4,7.4 10,7.4 C 9.1,7.4 8.5,7.9 8,8.5 C 7.5,7.9 6.9,7.4 6,7.4 Z"/>
  <path fill="#2e3436" d="M 8,0.5 C 4,0.5 1.2,3.3 1.2,7.4 L 1.2,8.4 L 2.6,8.4 L 2.6,7.4 C 2.6,4.1 4.8,1.9 8,1.9 C 11.2,1.9 13.4,4.1 13.4,7.4 L 13.4,8.4 L 14.8,8.4 L 14.8,7.4 C 14.8,3.3 12,0.5 8,0.5 Z"/>
  <rect fill="#2e3436" x="0.5" y="7.6" width="3" height="4.6" rx="1.2"/>
  <rect fill="#2e3436" x="12.5" y="7.6" width="3" height="4.6" rx="1.2"/>
  <circle fill="#2e3436" cx="6.6" cy="10" r="0.8"/>
  <circle fill="#2e3436" cx="9.4" cy="10" r="0.8"/>
  <path fill="#2e3436" d="M 6.9,11.1 L 9.1,11.1 L 8,12.5 Z"/>
</svg>
"""


def preview():
    """One row per icon: 128 on light and dark, then 64, 32, 24 and 16 px."""
    rows = []
    for label, name, symbolic in [("app icon", APP_ID, False), ("development build", f"{APP_ID}.Devel", False), ("symbolic", f"{APP_ID}-symbolic", True)]:
        cells = []
        for size, background in [(128, "#f6f5f4"), (128, "#241f31"), (64, "#f6f5f4"), (32, "#f6f5f4"), (24, "#f6f5f4"), (16, "#f6f5f4")]:
            if symbolic:
                picture = WORK / f"symbolic-{size}.png"
                subprocess.run(["rsvg-convert", "-w", str(size), "-h", str(size), "-o", str(picture), str(ICONS / f"{name}.svg")], check=True)
            else:
                picture = ICONS / "hicolor" / f"{size}x{size}" / "apps" / f"{name}.png"
            cell = WORK / f"cell-{name}-{size}-{background[1:]}.png"
            magick("-size", "160x160", f"xc:{background}", picture, "-gravity", "center", "-composite", cell)
            cells.append(cell)
        row = WORK / f"row-{name}.png"
        magick(*cells, "+append", row)
        labelled = WORK / f"labelled-{name}.png"
        magick(row, "-background", "#ffffff", "-font", FONT, "-pointsize", "20", "-fill", "#241f31", f"label:  {label}", "-gravity", "west", "-append", labelled)
        rows.append(labelled)
    magick(*rows, "-append", HERE / "preview-final.png")


def main():
    WORK.mkdir(exist_ok=True)
    tile = clean_tile()
    export_sizes(on_grid(tile, "app"), APP_ID)
    export_sizes(on_grid(with_devel_stripes(tile), "devel"), f"{APP_ID}.Devel")
    (ICONS / f"{APP_ID}-symbolic.svg").write_text(symbolic_icon())
    preview()
    print(HERE / "preview-final.png")


if __name__ == "__main__":
    main()
