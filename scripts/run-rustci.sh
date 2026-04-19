#!/usr/bin/env bash
set -euo pipefail

echo "[1/4] cargo fmt --all -- --check"
cargo fmt --all -- --check

echo "[2/4] cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings

echo "[3/4] cargo build --release"
cargo build --release

echo "[4/4] cargo test --all-features -- --nocapture"
cargo test --all-features -- --nocapture

echo "Rust CI checks passed."
