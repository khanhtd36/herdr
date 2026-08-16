---
name: fork-release
description: Cut a new khanhtd36/herdr fork release -- bump version, tag, push, watch CI build+publish+winget-submit. Use when the user asks to release a new version, cut a release, or bump the fork version.
---

# Fork release

Releasing `khanhtd36/herdr` (the fork, not upstream `herdrdev/herdr`) is a short local sequence; `.github/workflows/fork-release.yml` takes over from the pushed tag and does the rest unattended. Never reach for the upstream-only `just release` / `release-prepare` / `release-publish` recipes here -- their version regex (`^[0-9]+\.[0-9]+\.[0-9]+$`) rejects the `-khanhtd36.N` suffix and they drive the `v*`-tag `release.yml` pipeline, not this one.

## Steps

1. **Sync master.**
   ```
   git checkout master
   git pull origin master
   ```
   Working tree must be clean before continuing.

2. **Bump the version** in `Cargo.toml` (`version = "X.Y.Z-khanhtd36.N"`). Bump only `N` for a fork-only release. Reset `N` to `1` when `X.Y.Z` itself changed -- i.e. this release also picked up a new upstream sync.

3. **Refresh the lockfile and commit.**
   ```
   cargo update -p herdr --offline
   git add Cargo.toml Cargo.lock
   git commit -m "chore: bump version to X.Y.Z-khanhtd36.N"
   ```

4. **Push master, tag, push the tag.** Wrap every push with the `gh1` GitHub identity -- see gotcha below.
   ```
   GH_CONFIG_DIR="$HOME/.config/gh-khanhtd36" git push origin master
   git tag fork-vX.Y.Z-khanhtd36.N
   GH_CONFIG_DIR="$HOME/.config/gh-khanhtd36" git push origin fork-vX.Y.Z-khanhtd36.N
   ```

5. **Watch the triggered run to completion.**
   ```
   GH_CONFIG_DIR="$HOME/.config/gh-khanhtd36" gh run list --repo khanhtd36/herdr --workflow fork-release.yml --limit 1
   GH_CONFIG_DIR="$HOME/.config/gh-khanhtd36" gh run watch <run-id> --repo khanhtd36/herdr --exit-status
   ```
   Done when all six jobs -- `verify-version`, `build-linux`, `build-macos`, `build-windows`, `release`, `winget` -- report `success`. Confirm with:
   ```
   GH_CONFIG_DIR="$HOME/.config/gh-khanhtd36" gh release view fork-vX.Y.Z-khanhtd36.N --repo khanhtd36/herdr
   ```

## What CI already does -- don't duplicate it

`.github/workflows/fork-release.yml` triggers on the `fork-v*` tag and, unattended: verifies the tag matches `Cargo.toml`'s version, builds Linux x86_64, macOS aarch64, and Windows x86_64 (bare `.exe`, no ConPTY bundling -- matches this fork's distribution so far), creates the GitHub release on `khanhtd36/herdr`, and submits a winget-pkgs manifest update via `wingetcreate --submit` using the already-configured `WINGET_TOKEN` repo secret. No secrets or workflow edits are needed for a routine release.

## Gotchas

- **gh account**: the default `gh auth status` account (`khanhtd36arb`) does not have push/release access to this fork. Every `git push` / `gh` call above needs `GH_CONFIG_DIR="$HOME/.config/gh-khanhtd36"` -- the `gh1` PowerShell profile function does the same wrap interactively.
- **winget propagation lag**: the CI `winget` job submits a PR to the community `winget-pkgs` repo; it isn't merged/indexed instantly, so plain `winget install khanhtd36.herdr-khanhtd36` can lag the new release by hours. For immediate local testing, pull the previous version's 3 manifest files from `microsoft/winget-pkgs` as a template (`manifests/k/khanhtd36/herdr-khanhtd36/<prev-version>/`), point the installer YAML at the new release's `herdr-windows-x86_64.exe` asset URL + its SHA256, and `winget install --manifest <folder>`.
- **`just check` / `just test` can be unrelated-broken**: a pre-existing clippy failure (`persist/restore.rs` too-many-arguments) and a possible stale vendored-zig-lib cache aren't release blockers here -- CI's own build jobs are the correctness gate for this pipeline, not the full local check suite.
