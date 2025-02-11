# sqlite-hashes

[![GitHub](https://img.shields.io/badge/github-sqlite--hashes-8da0cb?logo=github)](https://github.com/nyurik/sqlite-hashes)
[![crates.io version](https://img.shields.io/crates/v/sqlite-hashes.svg)](https://crates.io/crates/sqlite-hashes)
[![docs.rs docs](https://docs.rs/sqlite-hashes/badge.svg)](https://docs.rs/sqlite-hashes)
[![crates.io version](https://img.shields.io/crates/l/sqlite-hashes.svg)](https://github.com/nyurik/sqlite-hashes/blob/main/LICENSE-APACHE)
[![CI build](https://github.com/nyurik/sqlite-hashes/actions/workflows/ci.yml/badge.svg)](https://github.com/nyurik/sqlite-hashes/actions)

Implement `SQLite` hashing functions with aggregation support, including MD5, SHA1, SHA224, SHA256, SHA384, SHA512,
FNV-1a, xxHash. Functions are available as a loadable extension, or as a Rust library.

See also [SQLite-compressions](https://github.com/nyurik/sqlite-compressions) extension for gzip, brotli, and bsdiff support.

## Usage

This `SQLite` extension adds hashing functions like `sha256(...)`, `sha256_hex(...)`, `sha256_concat`
and `sha256_concat_hex` for multiple hashing algorithms. The `sha256` and `sha256_concat` function returns a blob value,
while the `*_hex` return a HEX string similar to `SQLite`'s own `hex()` function.

Functions support any number of arguments, e.g. `sha256('foo', 'bar', 'baz')`, hashing them in order as if they were
concatenated. Functions can hash text and blob values, but will raise an error on other types like integers and floating
point numbers. All `NULL` values are ignored. When calling the built-in `SQLite`'s `hex(NULL)`, the result is an empty
string, so `sha256_hex(NULL)` will return an empty string as well to be consistent.

The `*_concat` functions support aggregate to compute combined hash over a set of values like a column in a table,
e.g. `sha256_concat` and `sha256_concat_hex`. Just like scalar functions, multiple arguments are also supported, so you
can compute a hash over a set of columns, e.g. `sha256_concat(col1, col2, col3)`.

**Note:** The window functionality is not supported in the loadable extension, only when used as as a Rust crate. PRs
welcome.

### Extension

To use as an extension, load the `libsqlite_hashes.so` shared library into `SQLite`.

```bash
$ sqlite3
sqlite> .load ./libsqlite_hashes
sqlite> SELECT md5_hex('Hello world!');
86FB269D190D2C85F6E0468CECA42A20
```

### Rust library

To use as a Rust library, add `sqlite-hashes` to your `Cargo.toml` dependencies. Then, register the needed functions
with `register_hash_functions(&db)`. This will register all available functions, or you can
use `register_md5_functions(&db)` or `register_sha256_functions(&db)` to register just the needed ones (you may also
disable the default features to reduce compile time and binary size).

```rust
use sqlite_hashes::{register_hash_functions, rusqlite::Connection};

// Connect to SQLite DB and register needed hashing functions
let db = Connection::open_in_memory().unwrap();
// can also use hash-specific ones like register_sha256_functions(&db)
register_hash_functions(&db).unwrap();

// Hash 'password' using SHA-256, and dump resulting BLOB as a HEX string
let sql = "SELECT hex(sha256('password'));";
let hash: String = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
assert_eq!(hash, "5E884898DA28047151D0E56F8DC6292773603D0D6AABBDD62A11EF721D1542D8");

// Same as above, but use sha256_hex() function to dump the result as a HEX string directly
let sql = "SELECT sha256_hex('password');";
let hash: String = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
assert_eq!(hash, "5E884898DA28047151D0E56F8DC6292773603D0D6AABBDD62A11EF721D1542D8");

// Hash 'pass' (as text) and 'word' (as blob) using SHA-256, and dump it as a HEX string
// The result is the same as the above 'password' example.
let sql = "SELECT sha256_hex(cast('pass' as text), cast('word' as blob));";
let hash: String = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
assert_eq!(hash, "5E884898DA28047151D0E56F8DC6292773603D0D6AABBDD62A11EF721D1542D8");
```

### Aggregate and Window Functions

When `aggregate` or `window` feature is enabled (default), there are functions to compute combined hash over a set of
values like a column in a table, e.g. `sha256_concat` and `sha256_concat_hex`. Just like scalar functions, multiple
arguments are also supported, so you can compute a hash over a set of columns, e.g. `sha256_concat(col1, col2, col3)`.
Note that the window functionality is not supported in the loadable extension.

#### IMPORTANT NOTE: ORDERING

`SQLite` does NOT guarantee the order of rows when executing aggregate functions. A
query `SELECT sha256_concat(v) FROM tbl ORDER BY v;` will NOT concatenate values in sorted order, but will use some
internal storage order instead.

`SQLite` [v3.44.0](https://www.sqlite.org/changes.html#version_3_44_0)(2023-11-01) added support for the
`ORDER BY` clause
**inside** the aggregate function call, e.g. `SELECT sha256_concat(v ORDER BY v) FROM tbl;`. Make sure to use that to
guarantee consistent results.

For older `SQLite` versions, one common workaround was to use a subquery,
e.g. `SELECT group_concat(v) FROM (SELECT v FROM tbl ORDER BY v);`. This is
NOT guaranteed to work in future versions of `SQLite`. See [discussion](https://sqlite.org/forum/info/a49d9c4083b5350c)
for more details.

Another way for older `SQLite` to guarantee the ordering is to use a window function.

```sql,ignore
SELECT sha256_concat_hex(v)
       OVER (ORDER BY v ROWS
           BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
FROM tbl
LIMIT 1;
```

The hashing window functions will only work if the starting point of the window is not moving (`UNBOUNDED PRECEDING`).
To force a non-NULL value, use COALESCE:

```sql,ignore
SELECT coalesce(
               (SELECT sha256_concat_hex(v)
                       OVER (ORDER BY v ROWS
                           BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
                FROM tbl
                LIMIT 1),
               sha256_hex('')
       );
```

Note that window functions are only available in `SQLite` 3.25 and later, so a bundled `SQLite` version must be used, at
least for now.

```rust
use sqlite_hashes::{register_hash_functions, rusqlite::Connection};

let db = Connection::open_in_memory().unwrap();
register_hash_functions(&db).unwrap();

// Pre-populate the DB with some data. Note that the b values are not alphabetical.
db.execute_batch("
    CREATE TABLE tbl(id INTEGER PRIMARY KEY, v TEXT);
    INSERT INTO tbl VALUES (1, 'bbb'), (2, 'ccc'), (3, 'aaa');
  ").unwrap();

let sql = "SELECT sha256_concat_hex(v) OVER (
    ORDER BY v ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
    FROM tbl LIMIT 1;";
let hash: String = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
assert_eq!(hash, "FB84A45F6DF7D1D17036F939F1CFEB87339FF5DBDF411222F3762DD76779A287");

// The above window aggregation example is equivalent to this scalar hash:
let sql = "SELECT sha256_hex('aaabbbccc');";
let hash: String = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
assert_eq!(hash, "FB84A45F6DF7D1D17036F939F1CFEB87339FF5DBDF411222F3762DD76779A287");
```

#### Using with `SQLx`

To use with [SQLx](https://crates.io/crates/sqlx), you need to get the raw handle from the
`SqliteConnection` and pass it to the registration function.

```rust,ignore
use rusqlite::Connection;
use sqlite_hashes::register_hash_functions;
use sqlx::sqlite::SqliteConnection;

async fn register_functions(sqlx_conn: &mut SqliteConnection) {
    // SAFETY: No query must be performed on `sqlx_conn` until `handle_lock` is dropped.
    let mut handle_lock = sqlx_conn.lock_handle().await.unwrap();
    let handle = handle_lock.as_raw_handle().as_ptr();

    // SAFETY: this is safe as long as handle_lock is valid.
    let rusqlite_conn = unsafe { Connection::from_handle(handle) }.unwrap();

    // Registration is attached to the connection, not to rusqlite_conn,
    // so it will be available for the entire lifetime of the `sqlx_conn`.
    // Registration will be automatically dropped when SqliteConnection is dropped.
    register_hash_functions(&rusqlite_conn).unwrap();
}
```

## Crate features

By default, this crate will compile with all features. You can enable just the ones you need to reduce compile time and
binary size.

```toml
[dependencies]
sqlite-hashes = { version = "0.8", default-features = false, features = ["hex", "window", "sha256"] }
```

* **trace** - enable tracing support, logging all function calls and their arguments
* **hex** - enable hex string functions like `*_hex()` and `*_concat_hex()` (if `aggregate` is enabled)
* **aggregate** - enable aggregate functions like `*_concat()` and `*_concat_hex()` (if `hex` is enabled)
* **window** - enable window functions support (implies `aggregate`)
* **md5** - enable MD5 hash support
* **sha1** - enable SHA1 hash support
* **sha224** - enable SHA224 hash support
* **sha256** - enable SHA256 hash support
* **sha384** - enable SHA384 hash support
* **sha512** - enable SHA512 hash support
* **fnv** - enable FNV-1a hash support
* **xxhash** - enable `xxh32, xxh64, xxh3_64, xxh3_128` hash support

The **`loadable_extension`** feature should only be used when building
a `.so` / `.dylib` / `.dll` extension file that can be loaded directly into sqlite3 executable.

## Development

* You must install `sqlite3` and `libsqlite3-dev`, e.g. `sudo apt install -y libsqlite3-dev sqlite3` on Ubuntu/Mint.
* This project is easier to develop with [just](https://github.com/casey/just#readme), a modern alternative to `make`.
  Install it with `cargo install just`.
* To get a list of available commands, run `just`.
* To run tests, use `just test`.
* On `git push`, it will run a few validations, including `cargo fmt`, `cargo clippy`, and `cargo test`.
  Use `git push --no-verify` to skip these checks.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
