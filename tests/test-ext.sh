#!/usr/bin/env bash
set -euo pipefail

SQLITE3_BIN=${SQLITE3_BIN:-sqlite3}
EXTENSION_FILE=${EXTENSION_FILE:-target/debug/examples/libsqlite_hashes}

if [ ! -f "$EXTENSION_FILE" ] && [ ! -f "$EXTENSION_FILE.so" ] && [ ! -f "$EXTENSION_FILE.dylib" ] && [ ! -f "$EXTENSION_FILE.dll" ]; then
    echo "Extension file $EXTENSION_FILE [.so|.dylib|.dll] do not exist. Run 'just build-ext' first. Available files:"
    ls -l "$EXTENSION_FILE"*
    exit 1
fi
echo "Using extension file '$EXTENSION_FILE [.so|.dylib|.dll]'"

if ! command -v "$SQLITE3_BIN" > /dev/null; then
    echo "$SQLITE3_BIN executable could not be found"
    exit 1
fi
echo "Found $SQLITE3_BIN executable $($SQLITE3_BIN --version)"

test_one() {
    local sql=$1
    local expected=$2

    echo "Trying to get  '$expected'  from  $sql"
    result=$($SQLITE3_BIN <<EOF
.log stderr
.load '$EXTENSION_FILE'
$sql
EOF
    )
    if [ "$result" != "$expected" ]; then
        echo "Failed SQL: $sql"
        echo "Expected:   $expected"
        echo "Actual:     $result"
        exit 1
    fi
}

test_hash() {
    local hash=$1
    local expected=$2
    test_one "SELECT hex(${hash}('12345'));"         "$expected"
    test_one "SELECT ${hash}_hex('12345');"          "$expected"
    test_one "SELECT hex(${hash}_concat('12345'));"  "$expected"
    test_one "SELECT ${hash}_concat_hex('12345');"   "$expected"
}

test_hash "md5"    "827CCB0EEA8A706C4C34A16891F84E7B"
test_hash "sha1"   "8CB2237D0679CA88DB6464EAC60DA96345513964"
test_hash "sha224" "A7470858E79C282BC2F6ADFD831B132672DFD1224C1E78CBF5BCD057"
test_hash "sha256" "5994471ABB01112AFCC18159F6CC74B4F511B99806DA59B3CAF5A9C173CACFC5"
test_hash "sha384" "0FA76955ABFA9DAFD83FACCA8343A92AA09497F98101086611B0BFA95DBC0DCC661D62E9568A5A032BA81960F3E55D4A"
test_hash "sha512" "3627909A29C31381A071EC27F7C9CA97726182AED29A7DDD2E54353322CFB30ABB9E3A6DF2AC2C20FE23436311D678564D0C8D305930575F60E2D3D048184D79"
test_hash "fnv1a"  "E575E8883C0F89F8"
test_hash "xxh32"  "B30D56B4"
test_hash "xxh64"  "C6F2D2DD0AD64FB6"
test_hash "xxh3_64"  "F34099EDE96B5581"
test_hash "xxh3_128"  "4AF3DA69F61E14CF26F4C14B6B6BFDB4"
