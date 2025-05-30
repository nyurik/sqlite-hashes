#![cfg_attr(feature = "default", doc = include_str!("../README.md"))]
//
// Unsafe code is required for cdylib, so only use it for this crate
#![forbid(unsafe_code)]

#[cfg(not(any(
    feature = "md5",
    feature = "sha1",
    feature = "sha224",
    feature = "sha256",
    feature = "sha384",
    feature = "sha512",
    feature = "blake3",
    feature = "fnv",
    feature = "xxhash",
)))]
compile_error!(
    "At least one of these features must be enabled: md5,sha1,sha224,sha256,sha384,sha512,blake3,fnv,xxhash"
);

/// Re-export of the [`rusqlite`](https://crates.io/crates/rusqlite) crate to avoid version conflicts.
pub use rusqlite;

use crate::rusqlite::{Connection, Result};

mod aggregate;

mod scalar;
pub use crate::scalar::NamedDigest;

mod state;
pub use crate::state::HashState;

#[cfg(feature = "md5")]
mod md5;

#[cfg(feature = "md5")]
pub use crate::md5::register_md5_functions;

#[cfg(feature = "sha1")]
mod sha1;

#[cfg(feature = "sha1")]
pub use crate::sha1::register_sha1_functions;

#[cfg(feature = "sha224")]
mod sha224;

#[cfg(feature = "sha224")]
pub use crate::sha224::register_sha224_functions;

#[cfg(feature = "sha256")]
mod sha256;

#[cfg(feature = "sha256")]
pub use crate::sha256::register_sha256_functions;

#[cfg(feature = "sha384")]
mod sha384;

#[cfg(feature = "sha384")]
pub use crate::sha384::register_sha384_functions;

#[cfg(feature = "sha512")]
mod sha512;

#[cfg(feature = "sha512")]
pub use crate::sha512::register_sha512_functions;

#[cfg(feature = "blake3")]
mod blake3;

#[cfg(feature = "blake3")]
pub use crate::blake3::register_blake3_functions;

#[cfg(feature = "fnv")]
mod fnv;

#[cfg(feature = "fnv")]
pub use crate::fnv::register_fnv_functions;

#[cfg(feature = "xxhash")]
mod xxhash;

#[cfg(feature = "xxhash")]
pub use crate::xxhash::register_xxhash_functions;

/// Register all hashing functions for the given `SQLite` connection.
/// This is a convenience function that calls all of the `register_*_function` functions.
/// Features must be enabled for the corresponding functions to be registered.
///
/// # Example
///
/// ```
/// # use sqlite_hashes::rusqlite::{Connection, Result};
/// # use sqlite_hashes::register_hash_functions;
/// # fn main() -> Result<()> {
/// let db = Connection::open_in_memory()?;
/// register_hash_functions(&db)?;
/// # if cfg!(all(feature = "hex", feature = "md5")) {
/// let hash: String = db.query_row("SELECT md5_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(&hash, "5D41402ABC4B2A76B9719D911017C592");
/// # }
/// # if cfg!(all(feature = "hex", feature = "sha1")) {
/// let hash: String = db.query_row("SELECT sha1_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(hash, "AAF4C61DDCC5E8A2DABEDE0F3B482CD9AEA9434D");
/// # }
/// # if cfg!(all(feature = "hex", feature = "sha224")) {
/// let hash: String = db.query_row("SELECT sha224_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(&hash, "EA09AE9CC6768C50FCEE903ED054556E5BFC8347907F12598AA24193");
/// # }
/// # if cfg!(all(feature = "hex", feature = "sha256")) {
/// let hash: String = db.query_row("SELECT sha256_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(&hash, "2CF24DBA5FB0A30E26E83B2AC5B9E29E1B161E5C1FA7425E73043362938B9824");
/// # }
/// # if cfg!(all(feature = "hex", feature = "sha384")) {
/// let hash: String = db.query_row("SELECT sha384_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(&hash, "59E1748777448C69DE6B800D7A33BBFB9FF1B463E44354C3553BCDB9C666FA90125A3C79F90397BDF5F6A13DE828684F");
/// # }
/// # if cfg!(all(feature = "hex", feature = "sha512")) {
/// let hash: String = db.query_row("SELECT sha512_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(hash, "9B71D224BD62F3785D96D46AD3EA3D73319BFBC2890CAADAE2DFF72519673CA72323C3D99BA5C11D7C7ACC6E14B8C5DA0C4663475C2E5C3ADEF46F73BCDEC043");
/// # }
/// # if cfg!(all(feature = "hex", feature = "blake3")) {
/// let hash: String = db.query_row("SELECT blake3_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(hash, "EA8F163DB38682925E4491C5E58D4BB3506EF8C14EB78A86E908C5624A67200F");
/// # }
/// # if cfg!(all(feature = "hex", feature = "fnv")) {
/// let hash: String = db.query_row("SELECT fnv1a_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(hash, "A430D84680AABD0B");
/// # }
/// # if cfg!(all(feature = "hex", feature = "xxhash")) {
/// let hash: String = db.query_row("SELECT xxh32_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(hash, "FB0077F9");
/// let hash: String = db.query_row("SELECT xxh64_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(hash, "26C7827D889F6DA3");
/// let hash: String = db.query_row("SELECT xxh3_64_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(hash, "9555E8555C62DCFD");
/// let hash: String = db.query_row("SELECT xxh3_128_hex('hello')", [], |r| r.get(0))?;
/// assert_eq!(hash, "B5E9C1AD071B3E7FC779CFAA5E523818");
/// # }
/// # Ok(())
/// # }
/// ```
pub fn register_hash_functions(conn: &Connection) -> Result<()> {
    #[cfg(feature = "md5")]
    register_md5_functions(conn)?;
    #[cfg(feature = "sha1")]
    register_sha1_functions(conn)?;
    #[cfg(feature = "sha224")]
    register_sha224_functions(conn)?;
    #[cfg(feature = "sha256")]
    register_sha256_functions(conn)?;
    #[cfg(feature = "sha384")]
    register_sha384_functions(conn)?;
    #[cfg(feature = "sha512")]
    register_sha512_functions(conn)?;
    #[cfg(feature = "blake3")]
    register_blake3_functions(conn)?;
    #[cfg(feature = "fnv")]
    register_fnv_functions(conn)?;
    #[cfg(feature = "xxhash")]
    register_xxhash_functions(conn)?;

    Ok(())
}
