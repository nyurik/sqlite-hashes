use noncrypto_digests::{Xxh32, Xxh3_128, Xxh3_64, Xxh64};

use crate::rusqlite::{Connection, Result};

/// Register `xxh32`, `xxh64`, `xxh3_64`, `xxh3_128`, `xxh3_64` SQL functions with the given `SQLite` connection.
/// The functions use [Rust xxHash implementation](https://github.com/DoumanAsh/xxhash-rust) to compute the hash of the argument(s) using zero as the seed value.
///
/// # Example
///
/// ```
/// # // Use Python to convert:
/// # //   print('"\\x' + '\\x'.join([f"{v:02X}" for v in [251, 0, 119, 249]])+'"')
/// # use sqlite_hashes::rusqlite::{Connection, Result};
/// # use sqlite_hashes::register_xxhash_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_xxhash_functions(&db)?;
/// let hash: Vec<u8> = db.query_row("SELECT xxh32('hello')", [], |r| r.get(0))?;
/// let expected = b"\xFB\x00\x77\xF9";
/// assert_eq!(hash, expected);
/// let hash: Vec<u8> = db.query_row("SELECT xxh64('hello')", [], |r| r.get(0))?;
/// let expected = b"\x26\xC7\x82\x7D\x88\x9F\x6D\xA3";
/// assert_eq!(hash, expected);
/// let hash: Vec<u8> = db.query_row("SELECT xxh3_64('hello')", [], |r| r.get(0))?;
/// let expected = b"\x95\x55\xE8\x55\x5C\x62\xDC\xFD";
/// assert_eq!(hash, expected);
/// let hash: Vec<u8> = db.query_row("SELECT xxh3_128('hello')", [], |r| r.get(0))?;
/// let expected = b"\xb5\xe9\xc1\xad\x07\x1b\x3e\x7f\xc7\x79\xcf\xaa\x5e\x52\x38\x18";
/// assert_eq!(hash, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_xxhash_functions(conn: &Connection) -> Result<()> {
    crate::scalar::create_hash_fn::<Xxh32>(conn, "xxh32")?;
    crate::scalar::create_hash_fn::<Xxh64>(conn, "xxh64")?;
    crate::scalar::create_hash_fn::<Xxh3_64>(conn, "xxh3_64")?;
    crate::scalar::create_hash_fn::<Xxh3_128>(conn, "xxh3_128")
}
