#!/bin/bash
# Run native verification sequentially, using the pinned toolchain and lockfile.
set -euxo pipefail
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"

cargo check --locked --workspace --all-targets
cargo build --locked --bin vaja_2
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings -W clippy::all
cargo test --locked --workspace --all-targets --all-features -- --test-threads=1
cargo test --locked --workspace --doc -- --test-threads=1
cargo test --locked --release --lib --all-features -- --test-threads=1
