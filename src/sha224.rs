use sha2::Sha224;

use crate::rusqlite::{Connection, Result};

/// Register the `sha224` SQL function with the given `SQLite` connection.
/// The function takes a single argument and returns the [SHA224 hash](https://en.wikipedia.org/wiki/SHA-2) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_hashes::rusqlite::{Connection, Result};
/// # use sqlite_hashes::register_sha224_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_sha224_functions(&db)?;
/// let hash: Vec<u8> = db.query_row("SELECT sha224('hello')", [], |r| r.get(0))?;
/// let expected = b"\xea\x09\xae\x9c\xc6\x76\x8c\x50\xfc\xee\x90\x3e\xd0\x54\x55\x6e\x5b\xfc\x83\x47\x90\x7f\x12\x59\x8a\xa2\x41\x93";
/// assert_eq!(hash, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_sha224_functions(conn: &Connection) -> Result<()> {
    crate::scalar::create_hash_fn::<Sha224>(conn, "sha224")
}
