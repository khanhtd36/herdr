# Herdr fork (khanhtd36)

Glossary for this fork's release and update language.

## Language

**Fork release**:
A published build of this fork, identified by a base version plus a fork counter, e.g. `0.9.5-khanhtd36.2`.
_Avoid_: preview, stable (this fork ships a single channel)

**Base version**:
The upstream `major.minor.patch` a fork release is built on.

**Fork counter**:
The number after `khanhtd36.` that orders fork releases sharing one base version. A release without one has counter 0.
_Avoid_: build number, revision

**Update manifest**:
The hosted `latest.json` that names the newest fork release, its binaries and their checksums.

**Managed install**:
A herdr binary owned by an external package manager (Homebrew, Nix, mise, winget). Herdr never replaces a managed install itself; it tells the user the package manager's command instead.
_Avoid_: system install

**Self-update**:
Herdr replacing its own binary with the newest fork release. Always confirmed by the user, never applies to a managed install.
