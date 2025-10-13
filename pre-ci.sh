#!/usr/bin/env bash

# Quick pre-CI check (Git hook exists anyway)
cargo clippy --all-targets && \
cargo build --release && \
wasm-pack build --target web --out-dir web/pkg && \
cargo test
