use md5::Md5;

use crate::core::create_hash_fn;
use crate::rusqlite::{Connection, Result};

/// Register the `md5` SQL function with the given `SQLite` connection.
/// The function takes a single argument and returns the [MD5 hash](https://en.wikipedia.org/wiki/MD5) (blob) of that argument.
/// The argument can be either a string or a blob.
/// If the argument is `NULL`, the result is `NULL`.
///
/// # Example
///
/// ```
/// # use sqlite_hashes::rusqlite::{Connection, Result};
/// # use sqlite_hashes::register_md5_function;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_md5_function(&db)?;
/// let hash: Vec<u8> = db.query_row("SELECT md5('hello')", [], |r| r.get(0))?;
/// let expected = b"\x5d\x41\x40\x2a\xbc\x4b\x2a\x76\xb9\x71\x9d\x91\x10\x17\xc5\x92";
/// assert_eq!(hash, expected);
/// # Ok(())
/// # }
/// ```
pub fn register_md5_function(conn: &Connection) -> Result<()> {
    create_hash_fn::<Md5>(conn, "md5")
}
