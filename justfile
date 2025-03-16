#!/usr/bin/env just --justfile

crate_name := 'sqlite_hashes'
sqlite3 := 'sqlite3'

@_default:
    just --list

# Clean all build artifacts
clean:
    cargo clean

# Update all dependencies, including breaking changes. Requires nightly toolchain (install with `rustup install nightly`)
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
    cargo msrv find --write-msrv --ignore-lockfile

build: build-lib build-ext

build-lib:
    cargo build --workspace

build-ext *ARGS:
    cargo build --example {{crate_name}} --no-default-features --features default_loadable_extension {{ARGS}}

cross-build-ext *ARGS:
    cross build --example {{crate_name}} --no-default-features --features default_loadable_extension {{ARGS}}

cross-build-ext-aarch64: (cross-build-ext "--target=aarch64-unknown-linux-gnu" "--release")

# Run cargo clippy to lint the code
clippy:
    cargo clippy --workspace --all-targets -- -D warnings
    cargo clippy --no-default-features --features default_loadable_extension -- -D warnings

# Test code formatting
test-fmt:
    cargo fmt --all -- --check

# Reformat all code `cargo fmt`. If nightly is available, use it for better results
fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v cargo +nightly &> /dev/null; then
        echo 'Reformatting Rust code using nightly Rust fmt to sort imports'
        cargo +nightly fmt --all -- --config imports_granularity=Module,group_imports=StdExternalCrate
    else
        echo 'Reformatting Rust with the stable cargo fmt.  Install nightly with `rustup install nightly` for better results'
        cargo fmt --all
    fi

# Build and open code documentation
docs:
    cargo doc --no-deps --open

# Quick compile without building a binary
check:
    RUSTFLAGS='-D warnings' cargo check --workspace --all-targets

# Quick compile - lib-only
check-lib:
    RUSTFLAGS='-D warnings' cargo check --workspace

# Generate code coverage report
coverage *ARGS="--no-clean --open":
    cargo llvm-cov --workspace --all-targets --include-build-script {{ARGS}}
    # TODO: add test coverage for the loadable extension too, and combine them
    # cargo llvm-cov --example {{crate_name}} --no-default-features --features default_loadable_extension --codecov --output-path codecov.info

# Generate code coverage report to upload to codecov.io
ci-coverage: && \
            (coverage '--codecov --output-path target/llvm-cov/codecov.info')
    # ATTENTION: the full file path above is used in the CI workflow
    mkdir -p target/llvm-cov

# Test the library
test *ARGS: \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,md5"      ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,sha1"     ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,sha224"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,sha256"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,sha384"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,sha512"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,fnv"      ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,xxhash"   ) \
    \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash"                      ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,aggregate"            ) \
    \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,hex"                  ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,hex,aggregate"        ) \
    \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,trace"                ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,trace,aggregate"      ) \
    \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,hex,trace"            ) \
    ( test-one-lib "--no-default-features" "--features" "md5,sha1,sha224,sha256,sha384,sha512,fnv,xxhash,hex,trace,aggregate"  ) \

test-ext: build-ext
    ./tests/test-ext.sh

cross-test-ext-aarch64:
    docker run \
            --rm \
            -v "$(pwd):/workspace" \
            -w /workspace \
            --entrypoint sh \
            -e EXTENSION_FILE=target/aarch64-unknown-linux-gnu/release/examples/lib{{crate_name}} \
            --platform linux/arm64 \
            arm64v8/ubuntu \
            -c 'apt-get update && apt-get install -y sqlite3 && tests/test-ext.sh'

[private]
test-one-lib *ARGS:
    @echo "### TEST {{ARGS}} #######################################################################################################################"
    RUSTDOCFLAGS="-D warnings" cargo test {{ARGS}}

# Test documentation
test-doc:
    RUSTDOCFLAGS="-D warnings" cargo test --doc
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Print Rust version information
@rust-info:
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

# Switch to the minimum rusqlite version
set-min-rusqlite-version: (assert "jq")
    #!/usr/bin/env bash
    set -eu
    MIN_RUSQL_VER="$(grep '^rusqlite =.*version = ">=' Cargo.toml | sed -E 's/.*version = "[^"0-9]*([0-9.-]+).*/\1/')"
    echo "Switching to minimum rusqlite version: $MIN_RUSQL_VER"
    cargo update -p rusqlite --precise "$MIN_RUSQL_VER"

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

# Ensure that a certain command is available
[private]
assert $COMMAND:
    @if ! type "{{COMMAND}}" > /dev/null; then \
        echo "Command '{{COMMAND}}' could not be found. Please make sure it has been installed on your computer." ;\
        exit 1 ;\
    fi
