use sha2::Sha512;

use crate::rusqlite::{Connection, Result};

/// Register the `sha512` SQL function with the given `SQLite` connection.
/// The function takes a single argument and returns the [SHA512 hash](https://en.wikipedia.org/wiki/SHA-2) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_hashes::rusqlite::{Connection, Result};
/// # use sqlite_hashes::register_sha512_function;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_sha512_function(&db)?;
/// let hash: Vec<u8> = db.query_row("SELECT sha512('hello')", [], |r| r.get(0))?;
/// let expected = b"\x9b\x71\xd2\x24\xbd\x62\xf3\x78\x5d\x96\xd4\x6a\xd3\xea\x3d\x73\x31\x9b\xfb\xc2\x89\x0c\xaa\xda\xe2\xdf\xf7\x25\x19\x67\x3c\xa7\x23\x23\xc3\xd9\x9b\xa5\xc1\x1d\x7c\x7a\xcc\x6e\x14\xb8\xc5\xda\x0c\x46\x63\x47\x5c\x2e\x5c\x3a\xde\xf4\x6f\x73\xbc\xde\xc0\x43";
/// assert_eq!(hash, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_sha512_function(conn: &Connection) -> Result<()> {
    crate::core::create_hash_fn::<Sha512>(conn, "sha512")
}
