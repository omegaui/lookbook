#!/usr/bin/env python3
"""Draws catalog/site/social-preview.jpg, the picture link previews show
(WhatsApp, Telegram, Discord, Mastodon…) when someone shares the app page.

    python3 catalog/make_social_image.py

It needs Pillow and the Adwaita Sans font. Run it again when the icon, the
tagline or the Explore screenshot changes. Link previews want 1200 × 630
pixels and, for WhatsApp, a file under about 300 KB.
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parent.parent
ICON = ROOT / "data/icons/hicolor/512x512/apps/io.github.omegaui.Syspedia.png"
SCREENSHOT = ROOT / "data/screenshots/explore.png"
OUTPUT = ROOT / "catalog/site/social-preview.jpg"
FONT = Path("/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf")

WIDTH, HEIGHT = 1200, 630
TOP_COLOR = (28, 53, 84)  # the metainfo's dark brand color, #1c3554
BOTTOM_COLOR = (20, 22, 34)
TITLE_COLOR = (255, 255, 255)
TAGLINE_COLOR = (207, 224, 247)
DIM_COLOR = (160, 176, 200)
# The Explore screenshot's content sits in the middle of a wide window.
SCREENSHOT_CONTENT = (340, 0, 1568, 1033)


def main():
    image = gradient_background()
    place_screenshot(image)
    place_icon(image)
    write_words(ImageDraw.Draw(image))
    image.convert("RGB").save(OUTPUT, "JPEG", quality=85, optimize=True, progressive=True)
    print(f"Wrote {OUTPUT.relative_to(ROOT)} ({OUTPUT.stat().st_size // 1024} KB)")


def gradient_background():
    image = Image.new("RGBA", (WIDTH, HEIGHT))
    draw = ImageDraw.Draw(image)
    for y in range(HEIGHT):
        share = y / (HEIGHT - 1)
        color = tuple(round(top + (bottom - top) * share) for top, bottom in zip(TOP_COLOR, BOTTOM_COLOR))
        draw.line([(0, y), (WIDTH, y)], fill=color)
    return image


def place_screenshot(image):
    """Straight on, running off the right and bottom edges, like the app's banners."""
    shot = Image.open(SCREENSHOT).convert("RGBA").crop(SCREENSHOT_CONTENT)
    width = 720
    shot = shot.resize((width, round(shot.height * width / shot.width)), Image.LANCZOS)
    left, top = 610, 150

    shadow = Image.new("RGBA", image.size, (0, 0, 0, 0))
    ImageDraw.Draw(shadow).rounded_rectangle(
        [left, top + 12, left + shot.width, top + 12 + shot.height], radius=22, fill=(0, 0, 0, 140)
    )
    image.alpha_composite(shadow.filter(ImageFilter.GaussianBlur(24)))

    mask = Image.new("L", shot.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, shot.width, shot.height], radius=22, fill=255)
    image.paste(shot, (left, top), mask)


def place_icon(image):
    icon = Image.open(ICON).convert("RGBA").resize((168, 168), Image.LANCZOS)
    image.alpha_composite(icon, (72, 70))


def write_words(draw):
    draw.text((80, 268), "Syspedia", font=font(88, 800), fill=TITLE_COLOR)
    draw.text((82, 376), "Find a Linux desktop", font=font(38, 600), fill=TAGLINE_COLOR)
    draw.text((82, 422), "that fits you", font=font(38, 600), fill=TAGLINE_COLOR)
    draw.text((82, 494), "Distros, desktops and compositors,", font=font(25, 400), fill=DIM_COLOR)
    draw.text((82, 528), "and what your own system can do", font=font(25, 400), fill=DIM_COLOR)
    draw.text((82, 576), "omegaui.github.io/syspedia", font=font(22, 600), fill=TAGLINE_COLOR)


def font(size, weight):
    """Adwaita Sans is a variable font with two axes: optical size (14 to
    32, larger for display text) and weight."""
    face = ImageFont.truetype(str(FONT), size)
    optical_size = max(14, min(32, size))
    face.set_variation_by_axes([optical_size, weight])
    return face


if __name__ == "__main__":
    main()
