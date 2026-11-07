# Rices

The Compositors page and each compositor's details page list rices: complete setups people share, with a bar, launcher, lock screen and theme that work together. Each entry links to its repository, where the install guide lives.

## Previews

Each preview is a picture or clip from the rice's own README, loaded live from where its author hosts it (GitHub or the project's site). Linux & You never bundles or mirrors them, and the cards credit the author. Pictures load when a card first appears on screen. Clips wait for the Play button, then download into memory and play muted on a loop, paused while the card is off screen.

If an author asks for their rice or preview to be removed, delete its entry from `rices` in `data/resources/catalog/catalog.json`.

## Current list

| Rice | Author | Runs on | Built with | License | Preview |
| --- | --- | --- | --- | --- | --- |
| [illogical-impulse](https://github.com/end-4/dots-hyprland) | end-4 | Hyprland | Quickshell | GPL-3.0 | image |
| [Caelestia](https://github.com/caelestia-dots/caelestia) | caelestia-dots | Hyprland | Quickshell | GPL-3.0 | video |
| [HyDE](https://github.com/HyDE-Project/HyDE) | HyDE Project | Hyprland | Waybar, Rofi | GPL-3.0 | image |
| [ML4W OS](https://github.com/mylinuxforwork/dotfiles) | mylinuxforwork | Hyprland | Waybar, Rofi | GPL-3.0 | image |
| [KooL's Hyprland Dots](https://github.com/JaKooLit/Hyprland-Dots) | JaKooLit | Hyprland | Waybar, Rofi | GPL-3.0 | video |
| [Noctalia](https://github.com/noctalia-dev/noctalia) | noctalia-dev | niri, Hyprland, sway, labwc | Quickshell | MIT | image |
| [Dux Shell](https://github.com/duxlabs-dev/dux-shell) | duxlabs-dev | niri, Hyprland, sway, labwc | Quickshell, Go | MIT | image |
| [iNiR](https://github.com/snowarch/iNiR) | snowarch | niri | Quickshell | GPL-3.0 | image |
| [Matshell](https://github.com/Neurarian/matshell) | Neurarian | Hyprland, river | AGS, Astal, GTK 4 | GPL-3.0 | image |
| [max-baz's dotfiles](https://github.com/max-baz/dotfiles) | max-baz | sway | Waybar | ISC | image |

## Adding one

Pick a rice that is maintained and popular, and use a picture from its own README (PNG or JPEG; the runtime may not decode WebP). A clip (`"kind": "video"`) works when the README has only a video. Check what each README says it supports before listing compositors; iNiR, for example, still contains Hyprland code but is only tested on niri.
