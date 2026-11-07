blueprints = custom_target(
  'blueprints',
  input: files(
    'ui/pages/distros_page.blp',
    'ui/pages/explore_page.blp',
    'ui/pages/item_detail_page.blp',
    'ui/widgets/catalog_card.blp',
    'ui/widgets/collection_card.blp',
    'ui/widgets/screenshot_card.blp',
    'ui/widgets/spotlight_card.blp',
    'ui/window.blp',
  ),
  output: '.',
  command: [
    find_program('blueprint-compiler'),
    'batch-compile',
    '@OUTPUT@',
    '@CURRENT_SOURCE_DIR@',
    '@INPUT@',
  ],
)

resources = gnome.compile_resources(
  'resources',
  'resources.gresource.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
  dependencies: blueprints,
)
