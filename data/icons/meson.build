# The app icon is artwork, so it ships as PNGs at every size GNOME looks for.
# .agentic-coding/design/icon/make_icon.py builds them from the source image.
foreach size : ['16', '24', '32', '48', '64', '128', '256', '512']
  folder = 'hicolor' / '@0@x@0@'.format(size) / 'apps'
  install_data(
    folder / '@0@.png'.format(application_id),
    install_dir: iconsdir / folder,
  )
endforeach

install_data(
  '@0@-symbolic.svg'.format(base_id),
  install_dir: iconsdir / 'hicolor' / 'symbolic' / 'apps',
  rename: '@0@-symbolic.svg'.format(application_id),
)
