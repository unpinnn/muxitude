# muxitude

`muxitude` is a Rust TUI package manager for Termux, inspired by Debian `aptitude`.

## Screenshot
<img width="700" alt="muxitude" src="https://github.com/user-attachments/assets/b29b142e-aca1-4bed-846d-9f30e79e4c10" />


## Description
`aptitude` does not run on Termux, so `muxitude` can be used as an alternative package manager for working with `pkg`/`apt`.

It is not a 1:1 reimplementation of `aptitude`, but it is already usable for daily flows:
- browse packages by groups/sections
- update package lists
- mark install/remove/hold/auto/manual actions
- review pending actions
- apply changes
- search (`/`, `n`, `N`)

## Shortcuts

The shortcuts below follow aptitude-style behavior for the features currently implemented:

- `u`: update package list
- `g`: review/apply pending actions
- `+`: mark install
- `-`: mark remove
- `:`: keep/clear mark
- `=`: hold
- `M`: mark auto
- `m`: mark manual
- `/`: find
- `n` / `N`: next / previous search match

## Build

```bash
cargo build --release
```

Binary:

```bash
./target/release/muxitude
```

## Install

### Option 1: Download prebuilt release assets (AArch64)

Download from GitHub Releases:

- `muxitude-<version>-aarch64-linux-android.tar.gz` (plain binary archive)
- `muxitude_<version>_aarch64.deb` (Termux/deb package)

For the tarball, extract and run:

```bash
tar -xzf muxitude-<version>-aarch64-linux-android.tar.gz
./muxitude
```

For the deb package on Termux:

```bash
apt install ./muxitude_<version>_aarch64.deb
```

### Option 2: Install from crates.io

```bash
cargo install muxitude
```

### Option 3: Install prebuilt binary with cargo-binstall

If you have `cargo-binstall` installed, you can install from GitHub release assets
without compiling from source:

```bash
cargo binstall muxitude
```

### Option 4: Install via npm (Termux aarch64)

The npm wrapper installs the prebuilt `.deb` release and lets `pkg` handle installation:

```bash
npm install -g muxitude
```

If your platform is not Termux on `linux/arm64`, it prints:
"No prebuilt binaries available for your platform."

## Optional section mapping merge

You can merge extra section mappings at runtime:

```bash
./muxitude --section-mappings-merge my-mappings.txt
```

Format:

```txt
package=section
```

Entries in the merge file override built-in mappings for the same package name.
