use std::panic::{RefUnwindSafe, UnwindSafe};

use digest::Digest;
#[cfg(feature = "trace")]
use log::trace;
use rusqlite::functions::Context;
use rusqlite::ToSql;

#[cfg(feature = "aggregate")]
use crate::aggregate::create_agg_function;
use crate::rusqlite::functions::FunctionFlags;
use crate::rusqlite::types::{Type, ValueRef};
use crate::rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};
use crate::rusqlite::{Connection, Result};
use crate::state::HashState;

#[cfg(not(feature = "trace"))]
macro_rules! trace {
    ($($arg:tt)*) => {};
}

pub trait NamedDigest: Digest {
    fn name() -> &'static str;
}

macro_rules! digest_names {
    ($($typ:ty => $name:literal),* $(,)?) => {
        digest_names!(
            $(
                $typ => $name @ $name,
            )*
        );
    };
    ($($typ:ty => $name:literal @ $feature:literal),* $(,)?) => {
        $(
            #[cfg(feature = $feature)]
            impl NamedDigest for $typ {
                fn name() -> &'static str {
                    $name
                }
            }
        )*
    };
}

digest_names! {
    md5::Md5 => "md5",
    sha1::Sha1 => "sha1",
    sha2::Sha224 => "sha224",
    sha2::Sha256 => "sha256",
    sha2::Sha384 => "sha384",
    sha2::Sha512 => "sha512",
}

digest_names! {
    noncrypto_digests::Fnv => "fnv1a" @ "fnv",
    noncrypto_digests::Xxh32 => "xxh32" @ "xxhash",
    noncrypto_digests::Xxh64 => "xxh64" @ "xxhash",
    noncrypto_digests::Xxh3_64 => "xxh3_64" @ "xxhash",
    noncrypto_digests::Xxh3_128 => "xxh3_128" @ "xxhash",
}

pub(crate) fn create_hash_fn<T: NamedDigest + Clone + UnwindSafe + RefUnwindSafe + 'static>(
    conn: &Connection,
    fn_name: &str,
) -> Result<()> {
    create_scalar_function(conn, fn_name, |c| {
        hash_fn::<T>(
            c,
            #[cfg(feature = "trace")]
            "",
        )
        .map(HashState::finalize)
    })?;

    #[cfg(feature = "hex")]
    {
        let fn_name = format!("{fn_name}_hex");
        create_scalar_function(conn, &fn_name, |c| {
            hash_fn::<T>(
                c,
                #[cfg(feature = "trace")]
                "_hex",
            )
            .map(HashState::finalize_hex)
        })?;
    }

    #[cfg(feature = "aggregate")]
    {
        let fn_name = format!("{fn_name}_concat");
        create_agg_function(
            conn,
            &fn_name,
            crate::aggregate::AggType::<T, Vec<u8>>::new(
                #[cfg(any(feature = "window", feature = "trace"))]
                &fn_name,
                #[cfg(feature = "window")]
                HashState::calc,
                HashState::finalize,
            ),
        )?;
    }

    #[cfg(all(feature = "aggregate", feature = "hex"))]
    {
        let fn_name = format!("{fn_name}_concat_hex");
        create_agg_function(
            conn,
            &fn_name,
            crate::aggregate::AggType::<T, String>::new(
                #[cfg(any(feature = "window", feature = "trace"))]
                &fn_name,
                #[cfg(feature = "window")]
                HashState::calc_hex,
                HashState::finalize_hex,
            ),
        )?;
    }

    Ok(())
}

pub fn create_scalar_function<F, T>(conn: &Connection, fn_name: &str, function: F) -> Result<()>
where
    F: Fn(&Context<'_>) -> Result<T> + Send + UnwindSafe + 'static,
    T: ToSql,
{
    trace!("Registering function {fn_name}");
    conn.create_scalar_function(
        fn_name,
        -1,
        FunctionFlags::SQLITE_UTF8
            | FunctionFlags::SQLITE_DETERMINISTIC
            | FunctionFlags::SQLITE_DIRECTONLY,
        function,
    )
}

fn hash_fn<T: NamedDigest + Clone + UnwindSafe + RefUnwindSafe + 'static>(
    ctx: &Context,
    #[cfg(feature = "trace")] suffix: &'static str,
) -> Result<HashState<T>> {
    let param_count = ctx.len();
    if param_count == 0 {
        return Err(InvalidParameterCount(param_count, 1));
    }
    let mut state = HashState::<T>::default();
    for idx in 0..param_count {
        let value = ctx.get_raw(idx);
        match value {
            ValueRef::Blob(val) => {
                trace!("{}{suffix}: hashing blob arg{idx}={val:?}", T::name());
                state.add_value(val);
            }
            ValueRef::Text(val) => {
                trace!("{}{suffix}: hashing text arg{idx}={val:?}", T::name());
                state.add_value(val);
            }
            ValueRef::Null => {
                trace!("{}{suffix}: ignoring arg{idx}=NULL", T::name());
                state.add_null();
            }
            #[allow(unused_variables)]
            ValueRef::Integer(val) => {
                trace!(
                    "{}{suffix}: unsupported Integer arg{idx}={val:?}",
                    T::name()
                );
                Err(InvalidFunctionParameterType(0, Type::Integer))?;
            }
            #[allow(unused_variables)]
            ValueRef::Real(val) => {
                trace!("{}{suffix}: unsupported Real arg{idx}={val:?}", T::name());
                Err(InvalidFunctionParameterType(0, Type::Real))?;
            }
        }
    }

    Ok(state)
}
