# Wallpaper sources

Research date: 2026-11-07.

The Wallpapers section pulls each source's live list when the user opens it, and loads images from the source's own servers. Lookbook never bundles, mirrors or re-hosts the images.

## Sources

| | orangc's walls | LinuxH2O |
| --- | --- | --- |
| Page | https://orangc.net/walls | https://linuxh2o.com/wallpapers/ |
| Live list | `https://files.orangc.net/media/walls/?ls` returns JSON from a copyparty file server: `files[]` with `href` (including an access key, for example `3d-model.png?k=qBA7`), `ext` and `sz`. 321 PNGs on 2026-11-07 | The page HTML. Each wallpaper is an `<img>` with a `srcset` of WordPress sizes. 113 images on 2026-11-07 |
| Thumbnail | Append `&th=j` to the file URL for a small JPEG (about 5 KB). `&th=w` gives WebP, which GTK cannot always decode | The `400w` entry of `srcset` (400×225) |
| Full image | The file URL, about 1.2 MB per PNG | The largest `srcset` entry |
| License | None. The README says the images were collected from many places on the internet, the collector made none of them, and artists can ask for removal | None stated. The page says the wallpapers were compiled, and invites contributions by email |

The older GitHub copy (https://github.com/orangci/walls, 333 images) is no longer the main location; its README points to orangc.net/walls.

Not usable as an API: the LinuxH2O WordPress media endpoint (`/wp-json/wp/v2/media?parent=2465`) returns only 5 of the images, because most were uploaded to other posts.

## Decision

Agreed on 2026-11-07: build the section now, load images straight from the sources with clear credit, and keep a takedown list in the catalog. The maintainer contacts both owners for permission. If either says no, its entry is removed from the catalog, with no app release needed.

## How the app uses them

- Each source is a catalog entry with a `kind` (`copyparty` or `wordpress-page`), so sources can be added, changed or removed through the catalog.
- `wallpaper_blocklist` in the catalog lists image URLs to hide, for takedown requests.
- Every request sends a User-Agent that names the app and links to its repository.
- Thumbnails load in pages of 48, so opening the section does not download hundreds of images at once.
- "Set as Wallpaper" uses the XDG Wallpaper portal. GNOME and KDE provide it; many tiling compositors' portal backends do not, so "Save" is always offered too.
