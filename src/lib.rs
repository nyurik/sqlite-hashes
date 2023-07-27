#[cfg(feature = "md5")]
mod md5;
#[cfg(feature = "md5")]
pub use crate::md5::create_fn_md5;

#[cfg(feature = "sha1")]
mod sha1;
#[cfg(feature = "sha1")]
pub use crate::sha1::create_fn_sha1;
