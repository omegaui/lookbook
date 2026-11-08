# Flathub manifest

`io.github.omegaui.Lookbook.json` is the manifest for the Flathub submission.
It builds with no network, as Flathub requires:

- the app comes from a `git` source pinned to a commit (point it at the
  release tag's commit, and add `"tag"`, when a release is tagged);
- the crates come from `cargo-sources.json`, which
  `build-aux/generate-cargo-sources.py` writes from `Cargo.lock`. Run it
  again whenever `Cargo.lock` changes.

The manifests next to `build-aux/` (`io.github.omegaui.Lookbook.json` and
the Devel one) build from the working folder and may download crates; they
are for local testing only.

Check it locally with `.agentic-coding/scripts/build-flathub.sh`, which
builds this manifest from the local repository instead of GitHub.
