#!/usr/bin/env just --justfile

sqlite3 := 'sqlite3'

@_default:
    just --list

# Clean all build artifacts
clean:
    cargo clean

# Update dependencies, including breaking changes
update:
    cargo +nightly -Z unstable-options update --breaking
    cargo update

# Find unused dependencies. Install it with `cargo install cargo-udeps`
udeps:
    cargo +nightly udeps --all-targets --workspace --all-features

# Check semver compatibility with prior published version. Install it with `cargo install cargo-semver-checks`
semver *ARGS:
    cargo semver-checks {{ARGS}}

# Find the minimum supported Rust version (MSRV) using cargo-msrv extension, and update Cargo.toml
msrv:
    cargo msrv find --write-msrv

build: build-lib build-ext

build-lib:
    cargo build --workspace

build-ext *ARGS:
    # Window is not supported because it requires bundling the SQLite library
    # See https://github.com/rusqlite/rusqlite/discussions/1423
    cargo build --example sqlite_hashes --no-default-features --features default_loadable_extension {{ARGS}}

cross-build-ext *ARGS:
    # Window is not supported because it requires bundling the SQLite library
    # See https://github.com/rusqlite/rusqlite/discussions/1423
    cross build --example sqlite_hashes --no-default-features --features default_loadable_extension {{ARGS}}

cross-build-ext-aarch64: (cross-build-ext "--target=aarch64-unknown-linux-gnu" "--release")

# Run cargo clippy
clippy:
    cargo clippy -- -D warnings
    cargo clippy --workspace --all-targets -- -D warnings
    cargo clippy --features aggregate -- -D warnings
    cargo clippy --features window -- -D warnings

# Test code formatting
test-fmt:
    cargo fmt --all -- --check

# Run cargo fmt
fmt:
    cargo +nightly fmt -- --config imports_granularity=Module,group_imports=StdExternalCrate

# Build and open code documentation
docs:
    cargo doc --no-deps --open

# Quick compile
check:
    RUSTFLAGS='-D warnings' cargo check --workspace --all-targets

# Quick compile - lib-only
check-lib:
    RUSTFLAGS='-D warnings' cargo check --workspace

# Test the library
test *ARGS: \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,md5"      ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,sha1"     ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,sha224"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,sha256"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,sha384"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,sha512"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,fnv"      ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,xxhash"   ) \
    \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash"                      ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,aggregate"            ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,window"               ) \
    \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,hex"                  ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,hex,aggregate"        ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,hex,window"           ) \
    \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,trace"                ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,trace,aggregate"      ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,trace,window"         ) \
    \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,hex,trace"            ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,hex,trace,aggregate"  ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,hex,trace,window"     ) \

test-ext: build-ext
    ./tests/test-ext.sh

cross-test-ext-aarch64:
    docker run \
            --rm \
            -v "$(pwd):/workspace" \
            -w /workspace \
            --entrypoint sh \
            -e EXTENSION_FILE=target/aarch64-unknown-linux-gnu/release/examples/libsqlite_hashes \
            --platform linux/arm64 \
            arm64v8/ubuntu \
            -c 'apt-get update && apt-get install -y sqlite3 && tests/test-ext.sh'

[private]
test-one-lib *ARGS:
    @echo "### TEST {{ARGS}} #######################################################################################################################"
    RUSTDOCFLAGS="-D warnings" cargo test {{ARGS}}

# Test documentation
test-doc:
    cargo test --doc
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

rust-info:
    rustc --version
    cargo --version

# Run all tests as expected by CI
ci-test: rust-info test-fmt clippy check test test-ext test-doc

# Run minimal subset of tests to ensure compatibility with MSRV
ci-test-msrv: rust-info check-lib test

[private]
is-sqlite3-available:
    #!/usr/bin/env sh
    set -eu
    if ! command -v {{sqlite3}} > /dev/null; then
        echo "{{sqlite3}} executable could not be found"
        exit 1
    fi
    echo "Found {{sqlite3}} executable:"
    {{sqlite3}} --version

# Run integration tests and save its output as the new expected output
bless *ARGS: (cargo-install "cargo-insta")
    cargo insta test --accept --unreferenced=auto {{ARGS}}

# Check if a certain Cargo command is installed, and install it if needed
[private]
cargo-install $COMMAND $INSTALL_CMD="" *ARGS="":
    #!/usr/bin/env sh
    set -eu
    if ! command -v $COMMAND > /dev/null; then
        if ! command -v cargo-binstall > /dev/null; then
            echo "$COMMAND could not be found. Installing it with    cargo install ${INSTALL_CMD:-$COMMAND} {{ARGS}}"
            cargo install ${INSTALL_CMD:-$COMMAND} {{ARGS}}
        else
            echo "$COMMAND could not be found. Installing it with    cargo binstall ${INSTALL_CMD:-$COMMAND} {{ARGS}}"
            cargo binstall ${INSTALL_CMD:-$COMMAND} {{ARGS}}
        fi
    fi

# Run benchmarks
bench:
    cargo bench
    open target/criterion/report/index.html

# Verify that the current version of the crate is not the same as the one published on crates.io
check-if-published:
    #!/usr/bin/env bash
    LOCAL_VERSION="$(grep '^version =' Cargo.toml | sed -E 's/version = "([^"]*)".*/\1/')"
    echo "Detected crate version:  $LOCAL_VERSION"
    CRATE_NAME="$(grep '^name =' Cargo.toml | head -1 | sed -E 's/name = "(.*)"/\1/')"
    echo "Detected crate name:     $CRATE_NAME"
    PUBLISHED_VERSION="$(cargo search ${CRATE_NAME} | grep "^${CRATE_NAME} =" | sed -E 's/.* = "(.*)".*/\1/')"
    echo "Published crate version: $PUBLISHED_VERSION"
    if [ "$LOCAL_VERSION" = "$PUBLISHED_VERSION" ]; then
        echo "ERROR: The current crate version has already been published."
        exit 1
    else
        echo "The current crate version has not yet been published."
    fi
