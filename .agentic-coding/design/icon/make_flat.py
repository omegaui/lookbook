#!/usr/bin/env python3
"""Flat app icon variants: a room corner seen from inside, drawn as a hexagon
split into three flat faces (left wall, right wall, floor), with at most one
small detail.

    python3 .agentic-coding/design/icon/make_flat.py

Writes flat/*.svg and preview-flat.png next to this script.
"""

import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "flat"
FONT = "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf"

BLUE = ["#99c1f1", "#62a0ea", "#3584e4", "#1c71d8", "#1a5fb4"]
YELLOW = ["#f9f06b", "#f8e45c", "#f6d32d", "#f5c211", "#e5a50a"]
ORANGE = ["#ffbe6f", "#ffa348", "#ff7800", "#e66100", "#c64600"]
PURPLE = ["#dc8add", "#c061cb", "#9141ac", "#813d9c", "#613583"]
LIGHT = ["#ffffff", "#f6f5f4", "#deddda", "#c0bfbc", "#9a9996"]
DARK = ["#77767b", "#5e5c64", "#3d3846", "#241f31", "#000000"]

# A pointy-top hexagon filling the 128 grid, and the corner where the walls meet the floor.
CENTER = (64, 66)
TOP = (64, 10)
UPPER_LEFT = (16, 38)
LOWER_LEFT = (16, 94)
BOTTOM = (64, 122)
LOWER_RIGHT = (112, 94)
UPPER_RIGHT = (112, 38)
CORNER_RADIUS = 9


def lerp(a, b, t):
    return (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)


def on_wall(origin, along, up, u0, u1, v0, v1):
    """A parallelogram on a wall: `along` and `up` are the wall's edge vectors."""
    def at(u, v):
        return (origin[0] + along[0] * u + up[0] * v, origin[1] + along[1] * u + up[1] * v)

    return [at(u0, v0), at(u1, v0), at(u1, v1), at(u0, v1)]


def polygon(points, fill):
    coords = " ".join(f"{x:.2f},{y:.2f}" for x, y in points)
    return f'<polygon points="{coords}" fill="{fill}"/>'


def rounded_hexagon():
    """The hexagon's outline with softly rounded corners, used as a clip."""
    corners = [TOP, UPPER_RIGHT, LOWER_RIGHT, BOTTOM, LOWER_LEFT, UPPER_LEFT]
    parts = []
    for index, corner in enumerate(corners):
        before = corners[index - 1]
        after = corners[(index + 1) % len(corners)]
        start = toward(corner, before, CORNER_RADIUS)
        end = toward(corner, after, CORNER_RADIUS)
        parts.append(f"{'M' if index == 0 else 'L'} {start[0]:.2f},{start[1]:.2f} Q {corner[0]},{corner[1]} {end[0]:.2f},{end[1]:.2f}")
    return " ".join(parts) + " Z"


def toward(point, target, distance):
    dx, dy = target[0] - point[0], target[1] - point[1]
    length = (dx * dx + dy * dy) ** 0.5
    return (point[0] + dx / length * distance, point[1] + dy / length * distance)


# Edge vectors of the two walls, from the corner's foot.
LEFT_ALONG = (UPPER_LEFT[0] - TOP[0], UPPER_LEFT[1] - TOP[1])   # toward the left edge
RIGHT_ALONG = (UPPER_RIGHT[0] - TOP[0], UPPER_RIGHT[1] - TOP[1])  # toward the right edge
UP = (TOP[0] - CENTER[0], TOP[1] - CENTER[1])                     # up the corner


def screen_on_right_wall(frame, glass):
    """A screen on the right wall."""
    return "\n".join(
        [
            polygon(on_wall(CENTER, RIGHT_ALONG, UP, 0.18, 0.82, 0.3, 0.75), frame),
            polygon(on_wall(CENTER, RIGHT_ALONG, UP, 0.25, 0.75, 0.37, 0.68), glass),
        ]
    )


def window_on_left_wall(glass):
    """A lit window on the left wall."""
    return polygon(on_wall(CENTER, LEFT_ALONG, UP, 0.3, 0.7, 0.32, 0.72), glass)


def icon(left_wall, right_wall, floor, detail=""):
    faces = "\n".join(
        [
            polygon([CENTER, TOP, UPPER_LEFT, LOWER_LEFT], left_wall),
            polygon([CENTER, TOP, UPPER_RIGHT, LOWER_RIGHT], right_wall),
            polygon([CENTER, LOWER_LEFT, BOTTOM, LOWER_RIGHT], floor),
            detail,
        ]
    )
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">\n'
        f'<defs><clipPath id="hexagon"><path d="{rounded_hexagon()}"/></clipPath></defs>\n'
        f'<g clip-path="url(#hexagon)">\n{faces}\n</g>\n</svg>\n'
    )


VARIANTS = {
    "1-screen": icon(PURPLE[2], PURPLE[3], ORANGE[1], screen_on_right_wall(DARK[3], BLUE[1])),
    "2-window": icon(PURPLE[2], PURPLE[3], ORANGE[1], window_on_left_wall(YELLOW[2])),
    "3-blue-room": icon(BLUE[2], BLUE[3], YELLOW[2], screen_on_right_wall(DARK[3], ORANGE[1])),
    "4-plain": icon(PURPLE[2], PURPLE[3], ORANGE[1]),
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
    subprocess.run(["magick", *rows, "-append", str(HERE / "preview-flat.png")], check=True)


def main():
    OUT.mkdir(exist_ok=True)
    for name, content in VARIANTS.items():
        (OUT / f"{name}.svg").write_text(content)
    render_preview()
    print(HERE / "preview-flat.png")


if __name__ == "__main__":
    main()
