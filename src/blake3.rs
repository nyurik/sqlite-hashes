use blake3::Hasher;

use crate::rusqlite::{Connection, Result};
use crate::scalar::create_hash_fn;

/// Register the `blake3` SQL function with the given `SQLite` connection.
/// The function takes a single argument and returns the [BLAKE3 hash](https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_hashes::rusqlite::{Connection, Result};
/// # use sqlite_hashes::register_blake3_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_blake3_functions(&db)?;
/// let hash: Vec<u8> = db.query_row("SELECT blake3('hello')", [], |r| r.get(0))?;
/// let expected = b"\xea\x8f\x16\x3d\xb3\x86\x82\x92\x5e\x44\x91\xc5\xe5\x8d\x4b\xb3\x50\x6e\xf8\xc1\x4e\xb7\x8a\x86\xe9\x08\xc5\x62\x4a\x67\x20\x0f";
/// assert_eq!(hash, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_blake3_functions(conn: &Connection) -> Result<()> {
    create_hash_fn::<Hasher>(conn, "blake3")
}
