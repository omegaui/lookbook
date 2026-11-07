#!/usr/bin/env python3
"""Draws the app icon concepts as 128x128 SVGs, then a preview sheet.

    python3 .agentic-coding/design/icon/make_concepts.py

Writes concepts/*.svg and preview.png next to this script. Needs rsvg-convert
and ImageMagick for the preview.
"""

import math
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "concepts"
FONT = "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf"

# GNOME palette
BLUE = ["#99c1f1", "#62a0ea", "#3584e4", "#1c71d8", "#1a5fb4"]
GREEN = ["#8ff0a4", "#57e389", "#33d17a", "#2ec27e", "#26a269"]
YELLOW = ["#f9f06b", "#f8e45c", "#f6d32d", "#f5c211", "#e5a50a"]
ORANGE = ["#ffbe6f", "#ffa348", "#ff7800", "#e66100", "#c64600"]
RED = ["#f66151", "#ed333b", "#e01b24", "#c01c28", "#a51d2d"]
PURPLE = ["#dc8add", "#c061cb", "#9141ac", "#813d9c", "#613583"]
BROWN = ["#cdab8f", "#b5835a", "#986a44", "#865e3c", "#63452c"]
LIGHT = ["#ffffff", "#f6f5f4", "#deddda", "#c0bfbc", "#9a9996"]
DARK = ["#77767b", "#5e5c64", "#3d3846", "#241f31", "#000000"]

COS30 = math.cos(math.radians(30))


class Iso:
    """Isometric projection: x runs down-right, y down-left, z up."""

    def __init__(self, origin_x, origin_y, scale=1.0):
        self.ox = origin_x
        self.oy = origin_y
        self.scale = scale

    def point(self, x, y, z=0):
        s = self.scale
        return (self.ox + (x - y) * COS30 * s, self.oy + (x + y) * 0.5 * s - z * s)

    def poly(self, points, fill, extra=""):
        coords = " ".join(f"{px:.2f},{py:.2f}" for px, py in (self.point(*p) for p in points))
        return f'<polygon points="{coords}" fill="{fill}" {extra}/>'

    def box(self, x0, y0, z0, x1, y1, z1, top, left, right):
        """A block: top face, then the two faces toward the viewer."""
        return "\n".join(
            [
                self.poly([(x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)], left),
                self.poly([(x1, y0, z0), (x1, y1, z0), (x1, y1, z1), (x1, y0, z1)], right),
                self.poly([(x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)], top),
            ]
        )


def svg(body, defs=""):
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">\n'
        f"<defs>{defs}</defs>\n{body}\n</svg>\n"
    )


def ground_shadow(cx, cy, rx, ry):
    return f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="#000" fill-opacity="0.18"/>'


def concept_corner():
    """A: your corner of Linux. A room corner with a glowing screen and a window."""
    iso = Iso(64, 52, 1.2)
    size, height, slab = 44, 38, 7
    parts = [ground_shadow(64, 117, 40, 6)]

    # Floor slab, seen from above, with its two front edges.
    parts.append(iso.poly([(0, size, 0), (size, size, 0), (size, size, -slab), (0, size, -slab)], BROWN[3]))
    parts.append(iso.poly([(size, 0, 0), (size, size, 0), (size, size, -slab), (size, 0, -slab)], BROWN[4]))
    parts.append(iso.poly([(0, 0, 0), (size, 0, 0), (size, size, 0), (0, size, 0)], BROWN[0]))

    # Two back walls, with a light top edge for thickness.
    parts.append(iso.poly([(0, 0, 0), (0, size, 0), (0, size, height), (0, 0, height)], PURPLE[3]))
    parts.append(iso.poly([(0, 0, 0), (size, 0, 0), (size, 0, height), (0, 0, height)], PURPLE[2]))
    parts.append(iso.poly([(0, 0, height), (0, size, height), (-3, size, height), (-3, -3, height)], PURPLE[1]))
    parts.append(iso.poly([(0, 0, height), (size, 0, height), (size, -3, height), (-3, -3, height)], PURPLE[0]))

    # A window on the left wall: an evening sky.
    parts.append(iso.poly([(0.3, 8, 13), (0.3, 30, 13), (0.3, 30, 31), (0.3, 8, 31)], DARK[3]))
    parts.append(iso.poly([(0.6, 10, 15), (0.6, 28, 15), (0.6, 28, 29), (0.6, 10, 29)], "url(#sky)"))

    # A screen on the right wall showing a tiling layout.
    parts.append(iso.poly([(6, 0.3, 12), (36, 0.3, 12), (36, 0.3, 32), (6, 0.3, 32)], DARK[3]))
    parts.append(iso.poly([(8, 0.6, 14), (21, 0.6, 14), (21, 0.6, 30), (8, 0.6, 30)], BLUE[2]))
    parts.append(iso.poly([(22, 0.6, 22.5), (34, 0.6, 22.5), (34, 0.6, 30), (22, 0.6, 30)], BLUE[1]))
    parts.append(iso.poly([(22, 0.6, 14), (34, 0.6, 14), (34, 0.6, 21.5), (22, 0.6, 21.5)], BLUE[0]))

    # A rug and a cushion: somewhere to sit.
    parts.append(iso.poly([(12, 12, 0.2), (36, 12, 0.2), (36, 36, 0.2), (12, 36, 0.2)], ORANGE[1]))
    parts.append(iso.box(26, 26, 0.2, 36, 36, 5, YELLOW[2], YELLOW[4], ORANGE[3]))

    defs = (
        '<linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">'
        f'<stop offset="0" stop-color="{BLUE[3]}"/><stop offset="1" stop-color="{ORANGE[1]}"/>'
        "</linearGradient>"
    )
    return svg("\n".join(parts), defs)


def concept_city():
    """B: window city. A tiling layout whose windows rise into blocks."""
    iso = Iso(64, 45, 1.2)
    size, slab = 52, 9
    parts = [ground_shadow(64, 118, 44, 6)]
    parts.append(iso.box(0, 0, -slab, size, size, 0, LIGHT[1], LIGHT[3], LIGHT[4]))

    blocks = [
        # x0, y0, x1, y1, height, (top, left, right)
        (4, 4, 24, 48, 30, (BLUE[1], BLUE[3], BLUE[4])),
        (28, 4, 48, 22, 20, (PURPLE[1], PURPLE[3], PURPLE[4])),
        (28, 26, 48, 48, 11, (ORANGE[1], ORANGE[3], ORANGE[4])),
    ]
    for x0, y0, x1, y1, height, (top, left, right) in blocks:
        parts.append(iso.box(x0, y0, 0, x1, y1, height, top, left, right))
        # A title bar along the back edge of each window's top.
        parts.append(iso.poly([(x0, y0, height), (x1, y0, height), (x1, y0 + 4, height), (x0, y0 + 4, height)], LIGHT[0], 'fill-opacity="0.55"'))
    return svg("\n".join(parts))


def concept_ampersand():
    """C: the "&" in Linux & You, chunky, with depth, on a GNOME-style tile."""
    amp = (
        "M 84,96 L 46,54 C 34,41 40,24 56,24 C 71,24 76,40 64,50 "
        "L 44,66 C 30,77 34,98 53,98 C 66,98 76,90 86,72"
    )
    parts = [
        '<rect x="12" y="18" width="104" height="100" rx="22" fill="#26a269"/>',
        '<rect x="12" y="12" width="104" height="100" rx="22" fill="url(#tile)"/>',
        f'<path d="{amp}" fill="none" stroke="{GREEN[4]}" stroke-width="15" stroke-linecap="round" stroke-linejoin="round" transform="translate(0 5)"/>',
        f'<path d="{amp}" fill="none" stroke="{LIGHT[0]}" stroke-width="15" stroke-linecap="round" stroke-linejoin="round"/>',
    ]
    defs = (
        '<linearGradient id="tile" x1="0" y1="0" x2="0" y2="1">'
        f'<stop offset="0" stop-color="{GREEN[1]}"/><stop offset="1" stop-color="{GREEN[3]}"/>'
        "</linearGradient>"
    )
    return svg("\n".join(parts), defs)


CONCEPTS = {
    "a-corner": concept_corner,
    "b-window-city": concept_city,
    "c-ampersand": concept_ampersand,
}


def render_preview():
    """One row per concept: 128 on light, 128 on dark, then 64, 32 and 16 px."""
    tmp = HERE / "concepts" / "render"
    tmp.mkdir(exist_ok=True)
    rows = []
    for name in CONCEPTS:
        source = OUT / f"{name}.svg"
        cells = []
        for size, background in [(128, "#f6f5f4"), (128, "#241f31"), (64, "#f6f5f4"), (32, "#f6f5f4"), (16, "#f6f5f4")]:
            png = tmp / f"{name}-{size}-{background[1:]}.png"
            subprocess.run(["rsvg-convert", "-w", str(size), "-h", str(size), "-o", str(png), str(source)], check=True)
            cell = tmp / f"cell-{png.name}"
            subprocess.run(
                ["magick", "-size", "160x160", f"xc:{background}", str(png), "-gravity", "center", "-composite", str(cell)],
                check=True,
            )
            cells.append(str(cell))
        row = tmp / f"row-{name}.png"
        subprocess.run(["magick", *cells, "+append", str(row)], check=True)
        labelled = tmp / f"labelled-{name}.png"
        subprocess.run(
            [
                "magick", str(row), "-background", "#ffffff", "-font", FONT, "-pointsize", "20",
                "-fill", "#241f31", f"label:  {name}", "-gravity", "west", "-append", str(labelled),
            ],
            check=True,
        )
        rows.append(str(labelled))
    subprocess.run(["magick", *rows, "-append", str(HERE / "preview.png")], check=True)


def main():
    OUT.mkdir(exist_ok=True)
    for name, draw in CONCEPTS.items():
        (OUT / f"{name}.svg").write_text(draw())
    render_preview()
    print(HERE / "preview.png")


if __name__ == "__main__":
    main()
