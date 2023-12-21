# sqlite-hashes

[![GitHub](https://img.shields.io/badge/github-sqlite--hashes-8da0cb?logo=github)](https://github.com/nyurik/sqlite-hashes)
[![crates.io version](https://img.shields.io/crates/v/sqlite-hashes.svg)](https://crates.io/crates/sqlite-hashes)
[![docs.rs docs](https://docs.rs/sqlite-hashes/badge.svg)](https://docs.rs/sqlite-hashes)
[![crates.io version](https://img.shields.io/crates/l/sqlite-hashes.svg)](https://github.com/nyurik/sqlite-hashes/blob/main/LICENSE-APACHE)
[![CI build](https://github.com/nyurik/sqlite-hashes/actions/workflows/ci.yml/badge.svg)](https://github.com/nyurik/sqlite-hashes/actions)


Use this crate to add various hash functions to SQLite, including MD5, SHA1, SHA224, SHA256, SHA384, and SHA512. 

This crate uses [rusqlite](https://crates.io/crates/rusqlite) to add user-defined functions using static linking. Eventually it would be good to build dynamically loadable extension binaries usable from other languages (PRs welcome).

## Usage

### Extension
To use as an extension, load the `sqlite_hashes.so` shared library into SQLite.

```bash
$ sqlite3
sqlite> .load sqlite_hashes.so
sqlite> SELECT md5_hex('Hello world!');
86FB269D190D2C85F6E0468CECA42A20
```

### Rust library

There are two types of scalar functions, the `<hash>(...)` and `<hash>_hex(...)`, e.g. `sha256(...)` and `sha256_hex(...)`. The first one returns a blob, and the second one returns a hex string.  All functions can hash text and blob values, but will raise an error on other types like integers and floating point numbers. Functions support any number of arguments, e.g. `sha256(a, b, c, ...)`, hashing them in order. All `NULL` values are ignored. When calling the built-in SQLite `hex(NULL)`, the result is an empty string, so `sha256_hex(NULL)` will return an empty string as well to be consistent.

```rust
use sqlite_hashes::{register_hash_functions, rusqlite::Connection};

fn main() {
    // Connect to SQLite DB and register needed hashing functions
    let db = Connection::open_in_memory().unwrap();
    // can also use hash-specific ones like register_sha256_function(&db)  
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
}
```

### Aggregate and Window Functions
When `aggregate` or `window` feature is enabled (default), there are functions to compute combined hash over a set of values like a column in a table, e.g. `sha256_concat` and `sha256_concat_hex`. Just like scalar functions, multiple arguments are also supported, so you can compute a hash over a set of columns, e.g. `sha256_concat(col1, col2, col3)`. Note that the window functionality is not supported in the loadable extension.

#### IMPORTANT NOTE: ORDERING

SQLite does NOT guarantee the order of rows when executing aggregate functions. A query `SELECT group_concat(v) FROM tbl ORDER BY v;` will NOT concatenate values in sorted order, but will use some internal storage order instead. Other databases like PostgreSQL support `SELECT string_agg(v ORDER BY v) FROM tbl;`, but SQLite does not.

One common workaround is to use a subquery, e.g. `SELECT group_concat(v) FROM (SELECT v FROM tbl ORDER BY v);`. This is NOT guaranteed to work in future versions of SQLite. See [discussion](https://sqlite.org/forum/info/a49d9c4083b5350c) for more details.

In order to guarantee the ordering, you must use a window function. 

```sql
SELECT sha256_concat_hex(v)
       OVER (ORDER BY v ROWS
             BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
FROM tbl
LIMIT 1;
```

The hashing window functions will only work if the starting point of the window is not moving (`UNBOUNDED PRECEDING`). To force a non-NULL value, use COALESCE:

```sql
SELECT coalesce(
    (SELECT sha256_concat_hex(v)
            OVER (ORDER BY v ROWS
                  BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
     FROM tbl
     LIMIT 1),
    sha256_hex('')
);
```

Note that window functions are only available in SQLite 3.25 and later, so a bundled SQLite version must be used, at least for now.

```rust 
use sqlite_hashes::{register_hash_functions, rusqlite::Connection};
fn main() {
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
}
```

## Crate features
By default, this crate will compile with all features. You can enable just the ones you need to reduce compile time and binary size.

```toml
[dependencies]
sqlite-hashes = { version = "0.6", default-features = false, features = ["hex", "window", "sha256"] }
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

## Development
* This project is easier to develop with [just](https://github.com/casey/just#readme), a modern alternative to `make`. Install it with `cargo install just`.
* To get a list of available commands, run `just`.
* To run tests, use `just test`.
* On `git push`, it will run a few validations, including `cargo fmt`, `cargo clippy`, and `cargo test`.  Use `git push --no-verify` to skip these checks.

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
