#!/usr/bin/env just --justfile

@_default:
    just --list --unsorted

# Clean all build artifacts
clean:
    cargo clean
    rm -f Cargo.lock

# Run cargo fmt and cargo clippy
lint: fmt clippy

# Run cargo fmt
fmt:
    cargo +nightly fmt -- --config imports_granularity=Module,group_imports=StdExternalCrate

# Run cargo clippy
clippy:
    cargo clippy --workspace --all-targets --bins --tests --lib --benches -- -D warnings

# Build and open code documentation
docs:
    cargo doc --no-deps --open

# Run all tests
test:
    ./.cargo-husky/hooks/pre-push

# Test documentation
test-doc:
    cargo test --doc

# Run integration tests and save its output as the new expected output
bless *ARGS: (cargo-install "insta" "cargo-insta")
    cargo insta test --accept --unreferenced=auto --all-features {{ ARGS }}

# Check if a certain Cargo command is installed, and install it if needed
[private]
cargo-install $COMMAND $INSTALL_CMD="" *ARGS="":
    @if ! command -v $COMMAND &> /dev/null; then \
        echo "$COMMAND could not be found. Installing it with    cargo install ${INSTALL_CMD:-$COMMAND} {{ ARGS }}" ;\
        cargo install ${INSTALL_CMD:-$COMMAND} {{ ARGS }} ;\
    fi
