#!/usr/bin/env python3
"""Writes build-aux/flathub/cargo-sources.json from Cargo.lock.

Flathub builds with no network, so every crate has to be a flatpak-builder
source. Each crates.io crate becomes an archive (checked by the sha256 in
Cargo.lock) unpacked into cargo/vendor, with the .cargo-checksum.json cargo
expects; a .cargo/config.toml in the source tree points cargo there. It is
the same layout flatpak-cargo-generator writes, for a lock file with only
crates.io dependencies, using only the standard library.

    python3 build-aux/generate-cargo-sources.py

Run it again whenever Cargo.lock changes.
"""

import json
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCK_FILE = ROOT / "Cargo.lock"
OUTPUT = ROOT / "build-aux/flathub/cargo-sources.json"
CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"
VENDOR_DIR = "cargo/vendor"
CARGO_CONFIG = f"""[source.vendored-sources]
directory = "{VENDOR_DIR}"

[source.crates-io]
replace-with = "vendored-sources"

[net]
offline = true
"""


def main():
    packages = tomllib.loads(LOCK_FILE.read_text())["package"]
    sources = []
    for package in packages:
        source = package.get("source")
        if source is None:
            continue  # the app itself
        if source != CRATES_IO:
            sys.exit(f"{package['name']} comes from {source}; only crates.io crates are supported")
        sources += crate_sources(package["name"], package["version"], package["checksum"])
    sources.append(cargo_config())

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(json.dumps(sources, indent=4) + "\n")
    print(f"Wrote {len(sources) // 2} crates to {OUTPUT.relative_to(ROOT)}")


def crate_sources(name, version, checksum):
    dest = f"{VENDOR_DIR}/{name}-{version}"
    return [
        {
            "type": "archive",
            "archive-type": "tar-gzip",
            "url": f"https://static.crates.io/crates/{name}/{name}-{version}.crate",
            "sha256": checksum,
            "dest": dest,
        },
        {
            "type": "inline",
            "contents": json.dumps({"package": checksum, "files": {}}),
            "dest": dest,
            "dest-filename": ".cargo-checksum.json",
        },
    ]


def cargo_config():
    """Cargo reads .cargo/config.toml from the folders above where it runs,
    and Meson runs it inside the source tree."""
    return {
        "type": "inline",
        "contents": CARGO_CONFIG,
        "dest": ".cargo",
        "dest-filename": "config.toml",
    }


if __name__ == "__main__":
    main()
