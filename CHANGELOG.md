# Changelog

All notable changes to this project are documented in this file.

## [0.0.5] - 2026-04-19

### Fixed
- npm installer now follows HTTP redirects when downloading `.deb` assets from GitHub Releases.

## [0.0.4] - 2026-04-19

### Fixed
- npm postinstall now correctly recognizes Termux when Node reports `process.platform=android`.
- npm install flow now supports both `android/arm64` and `linux/arm64` for Termux detection.

### Changed
- npm package name changed from `muxitude-cli` to `muxitude` for simpler install command (`npm install -g muxitude`).

## [0.0.3] - 2026-04-19

### Added
- GitHub Actions publish workflows:
  - `.github/workflows/publish-crates.yml`
  - `.github/workflows/publish-npm.yml`
- `scripts/set-repo-vars.sh` to set `CRATES_IO_TOKEN` and `NPM_TOKEN` from local `docs.1/*` token files.
- npm installer wrapper at repo root:
  - `package.json`
  - `scripts/npm-postinstall.js`

### Changed
- npm postinstall environment detection for Termux is now more robust (`PREFIX`, npm prefix, and `pkg` path checks).
- README install and shortcut sections were updated and clarified.

## [0.0.2] - 2026-04-19

### Added
- crates.io publish for `muxitude`.
- npm publish for `muxitude-cli`.
- GitHub release workflow now produces both:
  - `muxitude-<tag>-aarch64-linux-android.tar.gz`
  - `muxitude_<version>_aarch64.deb`
- GPL-3.0 licensing metadata and `LICENSE`.

### Changed
- Install documentation updated for tarball, `.deb`, cargo, and binstall paths.

## [0.0.1] - 2026-04-19

### Added
- Initial public release of `muxitude`.
- Termux-focused Rust TUI package manager with aptitude-style workflows.
