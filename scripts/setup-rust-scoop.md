# Setup Rust Toolchain With Scoop

If local Rust CI checks fail because `cargo fmt` or `cargo clippy` is missing, switch from Scoop's standalone Rust package to `rustup-gnu`.

## Why

- `scoop install rust` gives you `cargo`/`rustc`, but it may not expose Rust components like `rustfmt` and `clippy`.
- `rustup-gnu` gives you standard Rust toolchain management and lets you install CI components explicitly.

## Commands

Run these in PowerShell:

```powershell
scoop uninstall rust
scoop uninstall rust-gnu
scoop install rustup-gnu

# Open a new shell after install (recommended)
rustup set profile default
rustup default stable
rustup component add rustfmt clippy
```

## Verify

```powershell
cargo --version
rustc --version
cargo fmt --version
cargo clippy --version
```

If all of the above work, local Rust CI commands should run on this machine.
