# Ideas people build

Research date: 2026-11-07. Projects were checked live on extensions.gnome.org (EGO), GitHub, wayland.app, the wayland-explorer data files and project sources. r/unixporn could not be read; Reddit needs its OAuth API.

Link shorthand: `ego/N` is `https://extensions.gnome.org/extension/N`, `gh:` is `https://github.com/`. Ideas marked **⇄** have more than one way to build them. Download counts come from EGO; star counts from GitHub.

## Findings that change the app's design

1. **The Flatpak sandbox hides the protocols we want to detect.** Compositors hide "privileged" globals from sandboxed clients. sway does this in `is_privileged()` in `sway/server.c` (checked: it filters clients that have a `wp_security_context_v1` context, which Flatpak sets). niri does the same. Hidden globals include `zwlr_layer_shell_v1`, `ext_session_lock_manager_v1`, screencopy and image-copy-capture, data-control, gamma control, foreign-toplevel, output management, workspace and the virtual input protocols.

   So the app cannot list a compositor's features from inside Flatpak. Instead:
   - Identify the compositor from globals that are not hidden (`gtk_shell1` means Mutter, `org_kde_plasma_shell` means KWin, `hyprland_*` means Hyprland) together with `XDG_CURRENT_DESKTOP`.
   - Look up what that compositor supports in the catalog, which is built from wayland-explorer's per-compositor data (see [data-sources.md](data-sources.md)).
   - Show the globals the app actually saw separately, as "visible to apps".
2. **What GNOME offers.** Mutter 51 has no layer-shell, session-lock, screencopy, data-control or foreign-toplevel. It does have `ext_background_effect_manager_v1` (new in GNOME 51), `zwp_idle_inhibit_manager_v1` and `xdg_activation_v1`. On GNOME most ideas are built as Shell extensions, portals or D-Bus services.
3. **A Flatpak can install GNOME extensions.** Extension Manager does it through the `org.gnome.Shell.Extensions` D-Bus interface with `--talk-name=org.gnome.Shell.Extensions`. Lookbook could offer "Install" for featured extensions later.
4. **Projects move and get renamed often**, so example links must come from the live catalog, not the app's code. Changes found this year:
   - hyprexpo and hyprscrolling left hyprland-plugins in May 2026; scrolling is now built into Hyprland.
   - swww is now awww (Codeberg).
   - Krohnkite moved to Codeberg.
   - DankMaterialShell is now Dux Shell, and noctalia-shell is now noctalia.
   - grim moved to gitlab.freedesktop.org.
   - HyprPanel and pywal are archived (pywal16 continues pywal).

## GNOME

| Idea | How to build it | Examples | Difficulty |
| --- | --- | --- | --- |
| Dock ⇄ | GNOME: Shell extension (GJS). wlroots: layer-shell + foreign-toplevel. KDE: panel | Dash to Dock ego/307 (12.4M), gh:nwg-piotr/nwg-dock-hyprland | Medium |
| Tiling or scrolling windows ⇄ | GNOME: Shell extension. KDE: KWin script. Others: the compositor itself or a Hyprland plugin | PaperWM ego/6099, Tiling Shell ego/7065, gh:peterfajdiga/karousel | Hard |
| Blur ⇄ | GNOME: Shell extension, or apps use `ext_background_effect_manager_v1` (Mutter 51, KWin 6.7, niri). KDE: KWin effect | Blur my Shell ego/3193 (5.86M), gh:xarblu/kwin-effects-better-blur-dx | Medium |
| System monitor in the top bar ⇄ | GNOME: Shell extension. Others: bar module or plasmoid | Vitals ego/1460 (3.29M), Astra Monitor ego/6682 | Easy |
| Tray icons ⇄ | StatusNotifierItem + DBusMenu over D-Bus | AppIndicator Support ego/615 (3.25M) | Medium |
| Phone link ⇄ | KDE Connect protocol + D-Bus | GSConnect ego/1319 (2.82M), gh:andyholmes/valent | Hard |
| Window open and close effects ⇄ | GNOME: Shell extension (Clutter + GLSL). KDE: KWin effect. niri: custom shaders in config | Burn My Windows ego/4679 (1.36M), Magic Lamp ego/3740 | Medium |
| Now-playing island ⇄ | MPRIS over D-Bus, shown by an extension, layer-shell widget or plasmoid | gh:zickrian/isle, gh:ccatterina/plasmusic-toolbar | Easy to medium |
| Accent colors from the wallpaper ⇄ | GNOME: extension writing accent GSettings. Others: template generators | ChromaLeon ego/10070, gh:InioX/matugen | Easy |
| Top bar makeover | Shell extension | Just Perfection ego/3843 (2.2M), Open Bar ego/6580 | Easy |
| Search results in Activities | `org.gnome.Shell.SearchProvider2` over D-Bus; works from Flatpak | Dialect, Wike | Easy |
| A libadwaita app for GNOME Circle | GTK 4 + libadwaita, portals, Flathub | gh:AlexanderVanhee/Gradia, Bazaar, Resources | Medium |

## KDE Plasma

| Idea | How to build it | Examples | Difficulty |
| --- | --- | --- | --- |
| Panel widget (plasmoid) ⇄ | QML with the Plasma API, published on the KDE Store | gh:luisbocanegra/plasma-panel-colorizer, gh:antroids/application-title-bar | Easy to medium |
| Snap zones or auto-tiling | KWin script in JavaScript or QML | gh:gerritdevriese/kzones, gh:zeroxoneafour/polonium, Krohnkite (Codeberg) | Medium |
| Rounded corners ⇄ | KDE: KWin C++ effect, rebuilt per Plasma release. GNOME: extension. Hyprland and niri: config | gh:matinlotfali/KDE-Rounded-Corners, ego/7048 | Hard (KWin), medium |

## Wayland compositors (wlroots, Hyprland, niri, river)

| Idea | Protocols | Examples | Difficulty |
| --- | --- | --- | --- |
| Status bar ⇄ | `zwlr_layer_shell_v1`, plus `ext_workspace_manager_v1`, foreign-toplevel, compositor IPC | gh:Alexays/Waybar, gh:JakeStanger/ironbar, gh:elkowar/eww | Easy to medium |
| Full desktop shell | Quickshell (QML) or AGS/Astal, using layer-shell, session-lock, screencopy, toplevel | gh:caelestia-dots/shell, gh:noctalia-dev/noctalia | Hard |
| Launcher ⇄ | `zwlr_layer_shell_v1` with keyboard focus. GNOME: search provider. KDE: KRunner | fuzzel (Codeberg), gh:abenz1267/walker, gh:vicinaehq/vicinae | Easy |
| Notification daemon | `org.freedesktop.Notifications` + layer-shell | gh:emersion/mako, gh:ErikReider/SwayNotificationCenter | Medium |
| Lock screen ⇄ | `ext_session_lock_manager_v1` + PAM. Mutter and KWin do not offer it | gh:hyprwm/hyprlock, gh:jovanlanik/gtklock | Medium |
| Idle daemon or keep-awake ⇄ | `ext_idle_notifier_v1` (daemon), `zwp_idle_inhibit_manager_v1` (inhibit; Mutter has it) | gh:swaywm/swayidle, gh:hyprwm/hypridle, Caffeine ego/517 | Easy |
| Screenshot and annotate ⇄ | `ext_image_copy_capture_manager_v1` or `zwlr_screencopy_manager_v1`. GNOME and KDE: Screenshot portal | grim + slurp, gh:Satty-org/Satty, gh:hyprwm/hyprpicker | Easy |
| Screen recorder ⇄ | Screencopy or image-copy-capture. Any desktop: ScreenCast portal + PipeWire | gh:ammen99/wf-recorder, gh:SeaDve/Kooha | Medium |
| Clipboard history ⇄ | `zwlr_data_control_manager_v1` or `ext_data_control_manager_v1`. GNOME: extension | gh:sentriz/cliphist, Clipboard Indicator ego/779 | Easy |
| Wallpaper or live wallpaper ⇄ | Layer-shell background layer. GNOME: Hanabi extension. KDE: wallpaper plugin | awww (Codeberg), gh:GhostNaN/mpvpaper, gh:jeffshee/gnome-ext-hanabi | Easy to medium |
| Night light | `zwlr_gamma_control_manager_v1`, or `hyprland_ctm_control_manager_v1` | wlsunset, gammastep, gh:hyprwm/hyprsunset | Easy |
| Volume and brightness popup | Layer-shell + PipeWire or sysfs | gh:ErikReider/SwayOSD, gh:francma/wob | Easy |
| On-screen keyboard ⇄ | `zwp_virtual_keyboard_manager_v1` + `zwp_input_method_manager_v2` + layer-shell | gh:jjsullivan5196/wvkbd | Medium |
| Display profiles ⇄ | `zwlr_output_manager_v1`. GNOME: `org.gnome.Mutter.DisplayConfig`. KDE: `kde_output_management_v2` | kanshi, gh:nwg-piotr/nwg-displays | Medium |
| Hyprland plugin | C++ against Hyprland, installed with hyprpm | gh:outfoxxed/hy3, gh:KZDKM/Hyprspace | Hard |
| Your own river window manager | `river_window_manager_v1` (river 0.5) | tinyrwm (Codeberg) | Hard |
| libadwaita settings app for a compositor | GTK 4 + libadwaita over the compositor's config and IPC | gh:BlueManCZ/hyprmod, gh:nwg-piotr/nwg-look | Medium |

## Distros

| Idea | How to build it | Examples | Difficulty |
| --- | --- | --- | --- |
| Your own atomic OS image | bootc Containerfile or BlueBuild recipe, built by GitHub Actions | gh:ublue-os/image-template, gh:blue-build/template | Easy |
| Custom Arch ISO | archiso profile, built with `mkarchiso` | gh:archlinux/archiso | Medium |
| Fedora spin | Kickstart + `livemedia-creator` | pagure.io/fedora-kickstarts | Medium |
| NixOS flake config | `flake.nix` + home-manager | gh:Misterio77/nix-starter-configs | Medium to hard |
| Package and publish | PKGBUILD (AUR), spec (COPR), `debian/` (PPA), OBS | — | Easy to medium |
| Rice with a one-command installer | Dotfiles + install script + theme switcher | gh:end-4/dots-hyprland, gh:HyDE-Project/HyDE, gh:omacom/omarchy | Medium |

## What attracts attention

- **Motion beats still screenshots.** Top shells advertise animation, and window-effect extensions have over a million downloads each. Idea cards should support a short looping video.
- **The 2026 look is the morphing pill or "dynamic island".** It appears in isle, Tide Island, Dux Shell and many new Quickshell projects.
- **Cohesive theming**: one wallpaper change re-colors everything. A before/after slider would show this well.
- **Lock screens and look-alike themes** top the EGO popularity sort.
- **"Try it in five minutes"**: one-command setups (Omarchy, HyDE, end-4) get the most stars.
- For idea cards: name what the system offers ("ready because your compositor offers `zwlr_layer_shell_v1`"), offer the other route when one is missing ("Mutter has no layer-shell; build a Shell extension instead"), and show downloads or stars and a difficulty badge.

## Sources for trending ideas

- extensions.gnome.org: sorted by downloads, popularity or recent.
- GitHub topics, sorted by stars, filtered by `created:>` for new projects. Topic sizes on 2026-11-07: hyprland 7,371; quickshell 3,629; gnome-shell-extension 1,654; bluebuild 1,201; niri 926; plasmoid 451; kwin-script 94.
- Awesome lists: gh:hyprland-community/awesome-hyprland, gh:niri-wm/awesome-niri, gh:rcalixte/awesome-wayland, gh:Kazhnuz/awesome-gnome.
- apps.gnome.org for GNOME Circle apps.
- KDE Store through its OCS API (the web pages have a bot check).
- r/unixporn needs Reddit's OAuth API.
