#![allow(dead_code, unused_macros)]

use digest::Digest;
use insta::assert_snapshot;
use rusqlite::types::FromSql;
use rusqlite::{Connection, Result};

/// Simple hasher function that returns the hex-encoded hash of the input.
pub fn hash<T: Digest>(input: &[u8]) -> Vec<u8> {
    let mut hasher = T::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

/// Simple hasher function that returns the hex-encoded hash of the input.
pub fn hash_hex<T: Digest>(input: &[u8]) -> String {
    // Even though hex crate provides this functionality, its use is optional,
    // so we do it manually here to avoid test dependency on hex.
    let iter = hash::<T>(input).into_iter();
    iter.map(|b| format!("{:02X}", b)).collect()
}

/// Make sure the above hasher function produces the expected values,
/// and use it for validating the SQL functions.
#[test]
fn hasher() {
    #[cfg(feature = "md5")]
    assert_snapshot!(hash_hex::<md5::Md5>("test".as_bytes()), @"098F6BCD4621D373CADE4E832627B4F6");
    #[cfg(feature = "sha1")]
    assert_snapshot!(hash_hex::<sha1::Sha1>("test".as_bytes()), @"A94A8FE5CCB19BA61C4C0873D391E987982FBBD3");
    #[cfg(feature = "sha224")]
    assert_snapshot!(hash_hex::<sha2::Sha224>("test".as_bytes()), @"90A3ED9E32B2AAF4C61C410EB925426119E1A9DC53D4286ADE99A809");
    #[cfg(feature = "sha256")]
    assert_snapshot!(hash_hex::<sha2::Sha256>("test".as_bytes()), @"9F86D081884C7D659A2FEAA0C55AD015A3BF4F1B2B0B822CD15D6C15B0F00A08");
    #[cfg(feature = "sha384")]
    assert_snapshot!(hash_hex::<sha2::Sha384>("test".as_bytes()), @"768412320F7B0AA5812FCE428DC4706B3CAE50E02A64CAA16A782249BFE8EFC4B7EF1CCB126255D196047DFEDF17A0A9");
    #[cfg(feature = "sha512")]
    assert_snapshot!(hash_hex::<sha2::Sha512>("test".as_bytes()), @"EE26B0DD4AF7E749AA1A8EE3C10AE9923F618980772E473F8819A5D4940E0DB27AC185F8A0E1D5F84F88BC887FD67B143732C304CC5FA9AD8E6F57F50028A8FF");
}

/// Create macros like `md5!` asserting that first expression equals to the hash of the second one.
/// The macro evaluates to nothing if the corresponding feature is disabled.
/// The macro accepts the following syntax, comparing the `actual` value differently against
/// - `md5!(actual,   expected)` - expected string will be encoded as a hex string
/// - `md5!(actual, blob(expected))` - expected string will be encoded as a byte array
/// - `md5!(actual, bytes_as_blob(expected) )` - expected is a byte array to be encoded as a byte array
/// - `md5!(actual, bytes_as_hex(expected) )` - expected is a byte array to be encoded as a hex string
/// - `md5!(actual,   [ expected ])` - expected is a vector of strings to be encoded as a hex strings
/// - `md5!(actual, blob[ expected ])` - expected is a vector of strings to be encoded as a byte arrays
/// - `md5!(actual, bytes_as_blob[ expected ])` - expected is a vector of byte arrays to be encoded as a byte arrays
/// - `md5!(actual, bytes_as_hex[ expected ])` - expected is a vector of byte arrays to be encoded as a hex strings
macro_rules! hash_macros {
    ( $( $feat:literal $name:ident $typ:ty ),* $(,)? ) => {
        $(
            #[cfg(feature = $feat)]
            macro_rules! $name {
                ( $actual:expr, NULL ) => {{
                    let actual: rusqlite::Result<Option<Vec<u8>>> = $actual;
                    assert_eq!(actual, Ok(None), "asserting NULL result");
                }};
                ( $actual:expr, EMPTY ) => {{
                    let actual: rusqlite::Result<Option<String>> = $actual;
                    assert_eq!(actual, Ok(Some(String::from(""))), "asserting EMPTY result");
                }};
                ( $actual:expr, ERROR ) => {{
                    let actual: rusqlite::Result<Vec<u8>, rusqlite::Error> = $actual;
                    assert!(actual.is_err(), "asserting error result");
                }};
                ( $actual:expr, NO_ROWS ) => {{
                    let actual: rusqlite::Result<Vec<Vec<u8>>> = $actual;
                    assert!(actual.unwrap().is_empty(), "asserting NO_ROWS result");
                }};
                ( $actual:expr, blob[ $vec:expr ] ) => {{
                    let actual: rusqlite::Result<Vec<Vec<u8>>> = $actual;
                    assert_eq!(actual.unwrap(), $vec.iter().map(|v| $crate::utils::hash::<$typ>(v.as_bytes())).collect::<Vec<Vec<u8>>>())
                }};
                ( $actual:expr, bytes_as_blob[ $vec:expr ] ) => {{
                    let actual: rusqlite::Result<Vec<Vec<u8>>> = $actual;
                    assert_eq!(actual.unwrap(), $vec.iter().map(|v| $crate::utils::hash::<$typ>(v)).collect::<Vec<Vec<u8>>>())
                }};
                ( $actual:expr, bytes_as_hex[ $vec:expr ] ) => {{
                    let actual: rusqlite::Result<Vec<String>> = $actual;
                    assert_eq!(actual.unwrap(), $vec.iter().map(|v| $crate::utils::hash_hex::<$typ>(v)).collect::<Vec<Vec<u8>>>())
                }};
                ( $actual:expr, hex[ $vec:expr ] ) => {{
                    let actual: rusqlite::Result<Vec<String>> = $actual;
                    assert_eq!(actual.unwrap(), $vec.iter().map(|v| $crate::utils::hash_hex::<$typ>(v.as_bytes())).collect::<Vec<String>>())
                }};
                ( $actual:expr, blob($expected:expr) ) => {{
                    let actual: rusqlite::Result<Vec<u8>> = $actual;
                    assert_eq!(actual.unwrap(), $crate::utils::hash::<$typ>($expected.as_bytes()))
                }};
                ( $actual:expr, bytes_as_blob($expected:expr) ) => {{
                    let actual: rusqlite::Result<Vec<u8>> = $actual;
                    assert_eq!(actual.unwrap(), $crate::utils::hash::<$typ>($expected))
                }};
                ( $actual:expr, bytes_as_hex($expected:expr) ) => {{
                    let actual: rusqlite::Result<String> = $actual;
                    assert_eq!(actual.unwrap(), $crate::utils::hash_hex::<$typ>($expected))
                }};
                ( $actual:expr, hex($expected:expr) ) => {{
                    let actual: rusqlite::Result<String> = $actual;
                    assert_eq!(actual.unwrap(), $crate::utils::hash_hex::<$typ>($expected.as_bytes()))
                }};
            }
            #[cfg(not(feature = $feat))]
            macro_rules! $name {
                ($actual:expr, $exp:expr ) => {};
            }
        )*
    };
}

hash_macros!(
    "md5" md5 md5::Md5,
    "sha1" sha1 sha1::Sha1,
    "sha224" sha224 sha2::Sha224,
    "sha256" sha256 sha2::Sha256,
    "sha384" sha384 sha2::Sha384,
    "sha512" sha512 sha2::Sha512,
);

macro_rules! test_all {
    ( $conn:ident.$func:ident(*$suffix:tt), $($any:tt)* ) => {{
        let suffix = stringify!($suffix);
        test_all!( $conn.$func(suffix), $($any)* )
    }};
    ( $conn:ident.$func:ident($suffix:expr), $($any:tt)* ) => {{
        let suffix = $suffix;
        md5!( $conn.$func(&format!("md5{suffix}")), $($any)* );
        sha1!( $conn.$func(&format!("sha1{suffix}")), $($any)* );
        sha224!( $conn.$func(&format!("sha224{suffix}")), $($any)* );
        sha256!( $conn.$func(&format!("sha256{suffix}")), $($any)* );
        sha384!( $conn.$func(&format!("sha384{suffix}")), $($any)* );
        sha512!( $conn.$func(&format!("sha512{suffix}")), $($any)* );
    }};
}

pub struct Conn(Connection);

impl Conn {
    pub fn new() -> Self {
        let db = Connection::open_in_memory().unwrap();
        sqlite_hashes::register_hash_functions(&db).unwrap();
        db.execute_batch(
            "
CREATE TABLE tbl(id INTEGER PRIMARY KEY, v_text TEXT, v_blob BLOB, v_null_text TEXT, v_null_blob BLOB);
INSERT INTO tbl VALUES
        (1, 'bbb', cast('bbb' as BLOB), cast(NULL as TEXT), cast(NULL as BLOB)),
        (2, 'ccc', cast('ccc' as BLOB), cast(NULL as TEXT), cast(NULL as BLOB)),
        (3, 'aaa', cast('aaa' as BLOB), cast(NULL as TEXT), cast(NULL as BLOB));
",
        )
        .unwrap();
        Self(db)
    }

    pub fn sql<T: FromSql>(&self, query: &str) -> Result<T> {
        self.0.query_row_and_then(query, [], |r| r.get(0))
    }

    pub fn list<T: FromSql>(&self, query: &str) -> Result<Vec<T>> {
        let mut stmt = self.0.prepare(query).unwrap();
        stmt.query_map([], |row| row.get::<_, T>(0))
            .unwrap()
            .collect::<Result<Vec<T>>>()
    }

    pub fn select<T: FromSql>(&self, func: &str) -> Result<T> {
        self.sql(&format!("SELECT {func}"))
    }

    pub fn window_text_one<T: FromSql>(&self, func: &str) -> Result<T> {
        let sql = format!("SELECT {func}(v_text) OVER (ORDER BY v_text ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) FROM tbl LIMIT 1");
        self.sql(&sql)
    }

    pub fn window_text_zero<T: FromSql>(&self, func: &str) -> Result<Vec<T>> {
        let sql = format!("SELECT {func}(v_text) OVER (ORDER BY v_text ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) FROM tbl WHERE FALSE");
        self.list(&sql)
    }

    /// Should return hashes of `[aaa, aaabbb, aaabbbccc]`.
    pub fn growing_text_seq<T: FromSql>(&self, func: &str) -> Result<Vec<T>> {
        let sql = format!("SELECT {func}(v_text) OVER (ORDER BY v_text) FROM tbl");
        self.list(&sql)
    }

    /// This query moves the beginning of the window forward, so hash funcs will fail
    pub fn window_err<T: FromSql>(&self, func: &str) -> Result<Vec<T>> {
        let sql = format!("SELECT {func}(v_text) OVER (ORDER BY v_text ROWS 1 PRECEDING) FROM tbl");
        self.list(&sql)
    }

    /// The ordering here is un-documented and may change in the future.
    pub fn legacy_text_aggregate<T: FromSql>(&self, hash: &str) -> Result<T> {
        let sql = format!("SELECT {hash}(v_text) FROM (SELECT v_text FROM tbl ORDER BY v_text)");
        self.sql(&sql)
    }

    /// The ordering here is un-documented and may change in the future.
    pub fn legacy_blob_aggregate<T: FromSql>(&self, hash: &str) -> Result<T> {
        let sql = format!("SELECT {hash}(v_blob) FROM (SELECT v_blob FROM tbl ORDER BY v_text)");
        self.sql(&sql)
    }

    /// The ordering here is un-documented and may change in the future.
    pub fn legacy_null_text_aggregate<T: FromSql>(&self, hash: &str) -> Result<T> {
        let sql = format!(
            "SELECT {hash}(v_null_text) FROM (SELECT v_null_text FROM tbl ORDER BY v_text)"
        );
        self.sql(&sql)
    }

    /// The ordering here is un-documented and may change in the future.
    pub fn legacy_null_blob_aggregate<T: FromSql>(&self, hash: &str) -> Result<T> {
        let sql = format!(
            "SELECT {hash}(v_null_blob) FROM (SELECT v_null_blob FROM tbl ORDER BY v_text)"
        );
        self.sql(&sql)
    }

    /// Use RECURSIVE CTE to generate a sequence of numbers from 1 to `iterations`,
    pub fn sequence<T: FromSql>(&self, expr: &str, iterations: usize) -> Result<T> {
        // Modeled after https://stackoverflow.com/a/26241151/177275
        let sql = format!(
            "
WITH RECURSIVE
  seq(v) AS (
     SELECT 1
     UNION ALL
     SELECT v + 1 FROM seq
     LIMIT {iterations}
  )
SELECT {expr} FROM seq"
        );
        self.sql(&sql)
    }

    pub fn seq_0<T: FromSql>(&self, expr: &str) -> Result<T> {
        self.sequence(expr, 0)
    }

    pub fn seq_1<T: FromSql>(&self, expr: &str) -> Result<T> {
        self.sequence(expr, 1)
    }

    pub fn seq_1000<T: FromSql>(&self, expr: &str) -> Result<T> {
        self.sequence(expr, 1000)
    }
}
