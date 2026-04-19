# muxitude

`muxitude` is a Rust TUI package manager for Termux, inspired by Debian `aptitude`.

<img width="914" height="917" alt="image" src="https://github.com/user-attachments/assets/b29b142e-aca1-4bed-846d-9f30e79e4c10" />


It is not a 1:1 reimplementation of aptitude, but it is already usable for daily flows:
- browse packages by groups/sections
- update package lists
- mark install/remove/hold/auto/manual actions
- review pending actions
- apply changes
- search (`/`, `n`, `N`)

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
