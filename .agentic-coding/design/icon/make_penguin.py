#!/usr/bin/env python3
"""Flat penguin icon variants: an original, slim standing penguin with a scarf,
on snow or ice. Kept clearly apart from Tux (no chubby sitting pose, no yellow
feet, no white face mask).

    python3 .agentic-coding/design/icon/make_penguin.py

Writes penguin/*.svg and preview-penguin.png next to this script.
"""

import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "penguin"
FONT = "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf"

BLUE = ["#99c1f1", "#62a0ea", "#3584e4", "#1c71d8", "#1a5fb4"]
ORANGE = ["#ffbe6f", "#ffa348", "#ff7800", "#e66100", "#c64600"]
PURPLE = ["#dc8add", "#c061cb", "#9141ac", "#813d9c", "#613583"]
LIGHT = ["#ffffff", "#f6f5f4", "#deddda", "#c0bfbc", "#9a9996"]
DARK = ["#77767b", "#5e5c64", "#3d3846", "#241f31", "#000000"]
ICE = "#d6ecfb"


def penguin(dx=0, dy=0, scarf=PURPLE[2], scarf_tail=PURPLE[3]):
    """A slim standing penguin, centred at x = 64, feet at about y = 108."""
    return f"""<g transform="translate({dx} {dy})">
  <!-- flippers -->
  <ellipse cx="39" cy="74" rx="7" ry="20" fill="{DARK[2]}" transform="rotate(18 39 74)"/>
  <ellipse cx="89" cy="74" rx="7" ry="20" fill="{DARK[2]}" transform="rotate(-18 89 74)"/>
  <!-- body and belly -->
  <path d="M 64,18 C 84,18 90,40 90,62 C 90,90 82,108 64,108 C 46,108 38,90 38,62 C 38,40 44,18 64,18 Z" fill="{DARK[3]}"/>
  <path d="M 64,50 C 77,50 80,66 80,80 C 80,96 74,104 64,104 C 54,104 48,96 48,80 C 48,66 51,50 64,50 Z" fill="{LIGHT[0]}"/>
  <!-- eyes and beak -->
  <circle cx="55.5" cy="38" r="3.4" fill="{LIGHT[0]}"/>
  <circle cx="72.5" cy="38" r="3.4" fill="{LIGHT[0]}"/>
  <path d="M 58,44 L 70,44 L 64,51.5 Z" fill="{ORANGE[2]}" stroke="{ORANGE[2]}" stroke-width="2" stroke-linejoin="round"/>
  <!-- scarf -->
  <path d="M 41,55 C 52,61 76,61 87,55 L 87,63 C 76,69 52,69 41,63 Z" fill="{scarf}"/>
  <path d="M 72,62 L 80,62 L 82,82 L 73,80 Z" fill="{scarf_tail}"/>
  <!-- feet -->
  <ellipse cx="55" cy="108.5" rx="8" ry="3.5" fill="{ORANGE[3]}"/>
  <ellipse cx="73" cy="108.5" rx="8" ry="3.5" fill="{ORANGE[3]}"/>
</g>"""


def svg(body, defs=""):
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">\n'
        f"<defs>{defs}</defs>\n{body}\n</svg>\n"
    )


def on_snow():
    """The penguin on a snow drift."""
    snow = f"""
  <path d="M 10,112 C 30,98 98,98 118,112 C 118,120 110,124 64,124 C 18,124 10,120 10,112 Z" fill="{ICE}"/>
  <path d="M 10,110 C 30,96 98,96 118,110 C 108,115 20,115 10,110 Z" fill="{LIGHT[0]}"/>"""
    return svg(snow + penguin(dy=-4))


def on_ice_floe():
    """The penguin on an ice floe floating on water."""
    water = f'<path d="M 4,114 C 24,110 40,118 64,114 C 88,110 104,118 124,114 L 124,124 L 4,124 Z" fill="{BLUE[2]}"/>'
    floe = f"""
  <path d="M 18,104 L 110,104 L 104,118 L 24,118 Z" fill="{ICE}"/>
  <path d="M 14,100 L 114,100 L 110,106 L 18,106 Z" fill="{LIGHT[0]}"/>"""
    return svg(water + floe + penguin(dy=-8))


def in_night_badge():
    """The penguin on snow inside a round night-sky badge, with two stars."""
    defs = '<clipPath id="badge"><circle cx="64" cy="64" r="58"/></clipPath>'
    body = f"""
  <circle cx="64" cy="64" r="58" fill="{BLUE[4]}"/>
  <g clip-path="url(#badge)">
    <circle cx="30" cy="34" r="2.5" fill="{LIGHT[0]}"/>
    <circle cx="100" cy="28" r="2" fill="{LIGHT[0]}"/>
    <circle cx="104" cy="52" r="1.5" fill="{LIGHT[0]}"/>
    <path d="M 0,104 C 30,92 98,92 128,104 L 128,128 L 0,128 Z" fill="{LIGHT[0]}"/>
    <path d="M 0,114 C 40,106 88,106 128,114 L 128,128 L 0,128 Z" fill="{ICE}"/>
  </g>"""
    return svg(body + penguin(dy=-8, scarf="#f6d32d", scarf_tail="#e5a50a"), defs)


VARIANTS = {
    "p1-on-snow": on_snow(),
    "p2-on-ice-floe": on_ice_floe(),
    "p3-night-badge": in_night_badge(),
}


def render_preview():
    tmp = OUT / "render"
    tmp.mkdir(parents=True, exist_ok=True)
    rows = []
    for name in VARIANTS:
        source = OUT / f"{name}.svg"
        cells = []
        for size, background in [(128, "#f6f5f4"), (128, "#241f31"), (64, "#f6f5f4"), (32, "#f6f5f4"), (16, "#f6f5f4")]:
            png = tmp / f"{name}-{size}-{background[1:]}.png"
            subprocess.run(["rsvg-convert", "-w", str(size), "-h", str(size), "-o", str(png), str(source)], check=True)
            cell = tmp / f"cell-{png.name}"
            subprocess.run(["magick", "-size", "160x160", f"xc:{background}", str(png), "-gravity", "center", "-composite", str(cell)], check=True)
            cells.append(str(cell))
        row = tmp / f"row-{name}.png"
        subprocess.run(["magick", *cells, "+append", str(row)], check=True)
        labelled = tmp / f"labelled-{name}.png"
        subprocess.run(
            ["magick", str(row), "-background", "#ffffff", "-font", FONT, "-pointsize", "20", "-fill", "#241f31", f"label:  {name}", "-gravity", "west", "-append", str(labelled)],
            check=True,
        )
        rows.append(str(labelled))
    subprocess.run(["magick", *rows, "-append", str(HERE / "preview-penguin.png")], check=True)


def main():
    OUT.mkdir(exist_ok=True)
    for name, content in VARIANTS.items():
        (OUT / f"{name}.svg").write_text(content)
    render_preview()
    print(HERE / "preview-penguin.png")


if __name__ == "__main__":
    main()
