use sha2::Sha256;

use crate::rusqlite::{Connection, Result};

/// Register the `sha256` SQL function with the given `SQLite` connection.
/// The function takes a single argument and returns the [SHA256 hash](https://en.wikipedia.org/wiki/SHA-2) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_hashes::rusqlite::{Connection, Result};
/// # use sqlite_hashes::register_sha256_function;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_sha256_function(&db)?;
/// let hash: Vec<u8> = db.query_row("SELECT sha256('hello')", [], |r| r.get(0))?;
/// let expected = b"\x2c\xf2\x4d\xba\x5f\xb0\xa3\x0e\x26\xe8\x3b\x2a\xc5\xb9\xe2\x9e\x1b\x16\x1e\x5c\x1f\xa7\x42\x5e\x73\x04\x33\x62\x93\x8b\x98\x24";
/// assert_eq!(hash, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_sha256_function(conn: &Connection) -> Result<()> {
    crate::core::create_hash_fn::<Sha256>(conn, "sha256")
}
