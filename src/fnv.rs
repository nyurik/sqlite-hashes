use noncrypto_digests::Fnv;

use crate::rusqlite::{Connection, Result};

/// Register the `fnv1a` SQL function with the given `SQLite` connection.
/// The `fnv1a` function uses [Fowler–Noll–Vo hash function](https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function#FNV-1a_hash) to compute the hash of the argument(s).
///
/// # Example
///
/// ```
/// # use sqlite_hashes::rusqlite::{Connection, Result};
/// # use sqlite_hashes::register_fnv_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_fnv_functions(&db)?;
/// let hash: Vec<u8> = db.query_row("SELECT fnv1a('hello')", [], |r| r.get(0))?;
/// let expected = b"\xA4\x30\xD8\x46\x80\xAA\xBD\x0B";
/// assert_eq!(hash, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_fnv_functions(conn: &Connection) -> Result<()> {
    crate::scalar::create_hash_fn::<Fnv>(conn, "fnv1a")
}
