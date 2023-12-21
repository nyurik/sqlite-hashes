use noncrypto_digests::Fnv;

use crate::rusqlite::{Connection, Result};

/// Register the `fnv1a` SQL function with the given `SQLite` connection.
/// The function takes a single argument and returns the [SHA384 hash](https://en.wikipedia.org/wiki/SHA-2) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
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
