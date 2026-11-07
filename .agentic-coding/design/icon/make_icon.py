#!/usr/bin/env python3
"""Draws the Linux & You app icons: a slim penguin with a yellow scarf, standing
on snow inside a round night-sky badge. The penguin is our own and kept apart
from Tux (standing and slim, no white face mask, no yellow feet, a scarf).

    python3 .agentic-coding/design/icon/make_icon.py

Writes, in data/icons/:
- io.github.omegaui.LinuxAndYou.svg           the app icon (128x128)
- io.github.omegaui.LinuxAndYou.Devel.svg     the same with GNOME's development stripes on the ice
- io.github.omegaui.LinuxAndYou-symbolic.svg  a 16x16 one-color penguin

and preview-final.png next to this script. Earlier explorations: make_concepts.py
(isometric), make_flat.py (flat room corner), make_penguin.py (penguin layouts).
"""

import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent.parent
ICONS = ROOT / "data/icons"
APP_ID = "io.github.omegaui.LinuxAndYou"
FONT = "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf"

# GNOME palette, lightest to darkest.
BLUE = ["#99c1f1", "#62a0ea", "#3584e4", "#1c71d8", "#1a5fb4"]
YELLOW = ["#f9f06b", "#f8e45c", "#f6d32d", "#f5c211", "#e5a50a"]
ORANGE = ["#ffbe6f", "#ffa348", "#ff7800", "#e66100", "#c64600"]
LIGHT = ["#ffffff", "#f6f5f4", "#deddda", "#c0bfbc", "#9a9996"]
DARK = ["#77767b", "#5e5c64", "#3d3846", "#241f31", "#000000"]
ICE = "#d6ecfb"

# The penguin's outline, belly and flippers, in the 128 grid, feet at about y = 100.
BODY = "M 64,10 C 84,10 90,32 90,54 C 90,82 82,100 64,100 C 46,100 38,82 38,54 C 38,32 44,10 64,10 Z"
BELLY = "M 64,42 C 77,42 80,58 80,72 C 80,88 74,96 64,96 C 54,96 48,88 48,72 C 48,58 51,42 64,42 Z"
LEFT_FLIPPER = '<ellipse cx="39" cy="66" rx="7" ry="20" transform="rotate(18 39 66)"'
RIGHT_FLIPPER = '<ellipse cx="89" cy="66" rx="7" ry="20" transform="rotate(-18 89 66)"'


def penguin():
    return f"""
  {LEFT_FLIPPER} fill="{DARK[2]}"/>
  {RIGHT_FLIPPER} fill="{DARK[2]}"/>
  <path d="{BODY}" fill="{DARK[3]}"/>
  <path d="{BELLY}" fill="{LIGHT[0]}"/>
  <circle cx="55.5" cy="30" r="3.4" fill="{LIGHT[0]}"/>
  <circle cx="72.5" cy="30" r="3.4" fill="{LIGHT[0]}"/>
  <path d="M 58,36 L 70,36 L 64,43.5 Z" fill="{ORANGE[2]}" stroke="{ORANGE[2]}" stroke-width="2" stroke-linejoin="round"/>
  <path d="M 41,47 C 52,53 76,53 87,47 L 87,55 C 76,61 52,61 41,55 Z" fill="{YELLOW[2]}"/>
  <path d="M 72,54 L 80,54 L 82,74 L 73,72 Z" fill="{YELLOW[4]}"/>
  <ellipse cx="55" cy="100.5" rx="8" ry="3.5" fill="{ORANGE[3]}"/>
  <ellipse cx="73" cy="100.5" rx="8" ry="3.5" fill="{ORANGE[3]}"/>"""


def app_icon(devel=False):
    ice = "url(#devel-stripes)" if devel else ICE
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">
<defs>
  <clipPath id="badge"><circle cx="64" cy="64" r="58"/></clipPath>
  <pattern id="devel-stripes" width="10" height="10" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
    <rect width="5" height="10" fill="{YELLOW[2]}"/>
    <rect x="5" width="5" height="10" fill="{DARK[3]}"/>
  </pattern>
</defs>
<circle cx="64" cy="64" r="58" fill="{BLUE[4]}"/>
<g clip-path="url(#badge)">
  <circle cx="30" cy="34" r="2.5" fill="{LIGHT[0]}"/>
  <circle cx="100" cy="28" r="2" fill="{LIGHT[0]}"/>
  <circle cx="104" cy="52" r="1.5" fill="{LIGHT[0]}"/>
  <path d="M 0,96 C 30,84 98,84 128,96 L 128,128 L 0,128 Z" fill="{LIGHT[0]}"/>
  <path d="M 0,108 C 40,100 88,100 128,108 L 128,128 L 0,128 Z" fill="{ice}"/>
</g>{penguin()}
</svg>
"""


def symbolic_icon():
    """The penguin's silhouette with its belly cut out, scaled from the 128 grid."""
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
<g transform="translate(0 1.5) scale(0.125)" fill="#2e3436">
  <path fill-rule="evenodd" d="{BODY} {BELLY}"/>
  {LEFT_FLIPPER}/>
  {RIGHT_FLIPPER}/>
  <ellipse cx="55" cy="100.5" rx="9" ry="4.5"/>
  <ellipse cx="73" cy="100.5" rx="9" ry="4.5"/>
</g>
</svg>
"""


def render_preview(rows_to_draw):
    tmp = HERE / "render"
    tmp.mkdir(exist_ok=True)
    rows = []
    for label, source, sizes in rows_to_draw:
        cells = []
        for size, background in sizes:
            png = tmp / f"{source.stem}-{size}-{background[1:]}.png"
            subprocess.run(["rsvg-convert", "-w", str(size), "-h", str(size), "-o", str(png), str(source)], check=True)
            cell = tmp / f"cell-{png.name}"
            subprocess.run(["magick", "-size", "160x160", f"xc:{background}", str(png), "-gravity", "center", "-composite", str(cell)], check=True)
            cells.append(str(cell))
        row = tmp / f"row-{source.stem}.png"
        subprocess.run(["magick", *cells, "+append", str(row)], check=True)
        labelled = tmp / f"labelled-{source.stem}.png"
        subprocess.run(
            ["magick", str(row), "-background", "#ffffff", "-font", FONT, "-pointsize", "20", "-fill", "#241f31", f"label:  {label}", "-gravity", "west", "-append", str(labelled)],
            check=True,
        )
        rows.append(str(labelled))
    subprocess.run(["magick", *rows, "-append", str(HERE / "preview-final.png")], check=True)


def main():
    app = ICONS / f"{APP_ID}.svg"
    devel = ICONS / f"{APP_ID}.Devel.svg"
    symbolic = ICONS / f"{APP_ID}-symbolic.svg"
    app.write_text(app_icon())
    devel.write_text(app_icon(devel=True))
    symbolic.write_text(symbolic_icon())

    full = [(128, "#f6f5f4"), (128, "#241f31"), (64, "#f6f5f4"), (32, "#f6f5f4"), (16, "#f6f5f4")]
    small = [(128, "#f6f5f4"), (64, "#f6f5f4"), (32, "#f6f5f4"), (16, "#f6f5f4"), (16, "#ffffff")]
    render_preview([("app icon", app, full), ("development build", devel, full), ("symbolic", symbolic, small)])
    print(HERE / "preview-final.png")


if __name__ == "__main__":
    main()
