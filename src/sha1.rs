use sha1::Sha1;

use crate::rusqlite::{Connection, Result};

/// Register the `sha1` SQL function with the given `SQLite` connection.
/// The function takes a single argument and returns the [SHA1 hash](https://en.wikipedia.org/wiki/SHA-1) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_hashes::rusqlite::{Connection, Result};
/// # use sqlite_hashes::register_sha1_function;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_sha1_function(&db)?;
/// let hash: Vec<u8> = db.query_row("SELECT sha1('hello')", [], |r| r.get(0))?;
/// let expected = b"\xaa\xf4\xc6\x1d\xdc\xc5\xe8\xa2\xda\xbe\xde\x0f\x3b\x48\x2c\xd9\xae\xa9\x43\x4d";
/// assert_eq!(hash, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_sha1_function(conn: &Connection) -> Result<()> {
    crate::scalar::create_hash_fn::<Sha1>(conn, "sha1")
}
