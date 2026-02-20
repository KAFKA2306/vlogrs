# VLog Justfile

# --- General ---
default:
    @just --list

# --- Development ---
check:
    cargo check
    cargo clippy -- -A clippy::unwrap_used -D warnings

build:
    cargo build --release

test:
    cargo test

# --- Windows Agent ---
build-windows:
    cd src/windows/agent && cargo build --release --target x86_64-pc-windows-msvc

# --- Operations ---
start:
    cargo run -- monitor

setup:
    cargo run -- setup

status:
    cargo run -- status

# --- Housekeeping ---
clean:
    cargo clean
    rm -rf data/*.tmp
