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
    crate::core::create_hash_fn::<Sha1>(conn, "sha1")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::test::{hex, is_err, is_null, txt};

    #[test]
    fn test_sha1() {
        let db = Connection::open_in_memory().unwrap();
        let c = &db;
        register_sha1_function(c).unwrap();

        is_null(c, "sha1(NULL)");
        is_err(c, "sha1(1)");
        is_err(c, "sha1(0.42)");

        hex(c, "sha1('')", "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        hex(c, "sha1('a')", "86f7e437faa5a7fce15d1ddcb9eaeaea377667b8");
        hex(
            c,
            "sha1('123456789')",
            "f7c3bc1d808e04732adf679965ccc34ca7ae3441",
        );
        hex(c, "sha1(x'')", "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        hex(c, "sha1(x'00')", "5ba93c9db0cff93f52b521d7420e43f6eda2784f");
        hex(
            c,
            "sha1(x'0123456789abcdef')",
            "0ca2eadb529ac2e63abf9b4ae3df8ee121f10547",
        );

        txt(
            c,
            "hex(sha1(''))",
            "DA39A3EE5E6B4B0D3255BFEF95601890AFD80709",
        );
        txt(
            c,
            "hex(sha1('a'))",
            "86F7E437FAA5A7FCE15D1DDCB9EAEAEA377667B8",
        );
        txt(
            c,
            "hex(sha1(x'00'))",
            "5BA93C9DB0CFF93F52B521D7420E43F6EDA2784F",
        );
    }
}
