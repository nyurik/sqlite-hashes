#!/usr/bin/env just --justfile

sqlite3 := 'sqlite3'

@_default:
    just --list --unsorted

# Clean all build artifacts
clean:
    cargo clean

build: build-lib build-ext

build-lib:
    cargo build --workspace --all-targets --bins --tests --lib --benches

build-ext *ARGS:
    # Window is not supported because it requires bundling the SQLite library
    # See https://github.com/rusqlite/rusqlite/discussions/1423
    cargo  build --example sqlite_hashes --no-default-features --features default_loadable_extension {{ ARGS }}

cross-build-ext *ARGS:
    # Window is not supported because it requires bundling the SQLite library
    # See https://github.com/rusqlite/rusqlite/discussions/1423
    cross  build --example sqlite_hashes --no-default-features --features default_loadable_extension {{ ARGS }}

cross-build-ext-aarch64: (cross-build-ext "--target=aarch64-unknown-linux-gnu" "--release")

# Run cargo fmt and cargo clippy
lint: fmt clippy

# Run cargo fmt
fmt:
    cargo +nightly fmt -- --config imports_granularity=Module,group_imports=StdExternalCrate

# Run cargo clippy
clippy:
    cargo clippy -- -D warnings
    cargo clippy --workspace --all-targets --bins --tests --lib --benches --examples -- -D warnings
    cargo clippy --features aggregate -- -D warnings
    cargo clippy --features window -- -D warnings

# Build and open code documentation
docs:
    cargo doc --no-deps --open

# Test documentation
test-doc:
    cargo test --doc
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Run all tests
test:
    rustc --version
    cargo --version
    cargo fmt --all -- --check
    RUSTFLAGS='-D warnings' cargo build
    {{ just_executable() }} test-lib
    @echo "### DOCS #######################################################################################################################"
    {{ just_executable() }} test-doc
    @echo "### CLIPPY #####################################################################################################################"
    {{ just_executable() }} clippy
    @echo "### BUILD EXTENSION ############################################################################################################"
    {{ just_executable() }} build-ext
    @echo "### TEST EXTENSION #############################################################################################################"
    {{ just_executable() }} test-ext

# Test the library
test-lib *ARGS: \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,md5"      ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,sha1"     ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,sha224"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,sha256"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,sha384"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,sha512"   ) \
    ( test-one-lib "--no-default-features" "--features" "trace,hex,window,fnv"   ) \
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

test-ext:
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
    @echo "### TEST {{ ARGS }} #######################################################################################################################"
    cargo test {{ ARGS }}

[private]
is-sqlite3-available:
    #!/usr/bin/env sh
    set -eu
    if ! command -v {{ sqlite3 }} &> /dev/null; then
        echo "{{ sqlite3 }} executable could not be found"
        exit 1
    fi
    echo "Found {{ sqlite3 }} executable:"
    {{ sqlite3 }} --version

# Run integration tests and save its output as the new expected output
bless *ARGS: (cargo-install "insta" "cargo-insta")
    cargo insta test --accept --unreferenced=auto {{ ARGS }}

# Check if a certain Cargo command is installed, and install it if needed
[private]
cargo-install $COMMAND $INSTALL_CMD="" *ARGS="":
    #!/usr/bin/env sh
    set -eu
    if [ ! command -v $COMMAND &> /dev/null ]; then
        echo "$COMMAND could not be found. Installing it with    cargo install ${INSTALL_CMD:-$COMMAND} {{ ARGS }}"
        cargo install ${INSTALL_CMD:-$COMMAND} {{ ARGS }}
    fi
