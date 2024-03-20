use sha2::Sha384;

use crate::rusqlite::{Connection, Result};

/// Register the `sha384` SQL function with the given `SQLite` connection.
/// The function takes a single argument and returns the [SHA384 hash](https://en.wikipedia.org/wiki/SHA-2) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_hashes::rusqlite::{Connection, Result};
/// # use sqlite_hashes::register_sha384_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_sha384_functions(&db)?;
/// let hash: Vec<u8> = db.query_row("SELECT sha384('hello')", [], |r| r.get(0))?;
/// let expected = b"\x59\xe1\x74\x87\x77\x44\x8c\x69\xde\x6b\x80\x0d\x7a\x33\xbb\xfb\x9f\xf1\xb4\x63\xe4\x43\x54\xc3\x55\x3b\xcd\xb9\xc6\x66\xfa\x90\x12\x5a\x3c\x79\xf9\x03\x97\xbd\xf5\xf6\xa1\x3d\xe8\x28\x68\x4f";
/// assert_eq!(hash, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_sha384_functions(conn: &Connection) -> Result<()> {
    crate::scalar::create_hash_fn::<Sha384>(conn, "sha384")
}
