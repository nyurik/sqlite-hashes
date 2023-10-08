use std::panic::{RefUnwindSafe, UnwindSafe};

use digest::Digest;
#[cfg(feature = "hex")]
use hex::ToHex as _;
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

#[cfg(not(feature = "trace"))]
macro_rules! trace {
    ($($arg:tt)*) => {};
}

pub(crate) fn create_hash_fn<T: Digest + Clone + UnwindSafe + RefUnwindSafe + 'static>(
    conn: &Connection,
    fn_name: &str,
) -> Result<()> {
    create_scalar_function(conn, fn_name, hash_fn::<T>)?;

    #[cfg(feature = "hex")]
    {
        let fn_name = format!("{fn_name}_hex");
        create_scalar_function(conn, &fn_name, |c| to_hex(hash_fn::<T>(c)))?;
    }

    #[cfg(feature = "aggregate")]
    {
        use crate::aggregate::AggType;
        let fn_name = format!("{fn_name}_concat");
        create_agg_function(
            conn,
            &fn_name,
            AggType::<T>::new(
                #[cfg(any(feature = "window", feature = "trace"))]
                fn_name.clone(),
            ),
        )?;
    }

    #[cfg(all(feature = "aggregate", feature = "hex"))]
    {
        use crate::aggregate::AggHexType;
        let fn_name = format!("{fn_name}_concat_hex");
        create_agg_function(
            conn,
            &fn_name,
            AggHexType::<T>::new(
                #[cfg(any(feature = "window", feature = "trace"))]
                fn_name.clone(),
            ),
        )?;
    }

    Ok(())
}

pub fn create_scalar_function<F, T>(conn: &Connection, fn_name: &str, function: F) -> Result<()>
where
    F: FnMut(&Context<'_>) -> Result<T> + Send + UnwindSafe + 'static,
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

fn hash_fn<T: Digest + Clone + UnwindSafe + RefUnwindSafe + 'static>(
    ctx: &Context,
) -> Result<Option<Vec<u8>>> {
    let param_count = ctx.len();
    if param_count == 0 {
        return Err(InvalidParameterCount(param_count, 1));
    }
    let mut digest = T::new();
    let mut has_vals = false;
    for idx in 0..param_count {
        let value = ctx.get_raw(idx);
        match value {
            ValueRef::Blob(val) => {
                trace!("hashing blob arg{idx}={val:?}");
                digest.update(val);
                has_vals = true;
            }
            ValueRef::Text(val) => {
                trace!("hashing text arg{idx}={val:?}");
                digest.update(val);
                has_vals = true;
            }
            ValueRef::Null => {
                trace!("ignoring NULL");
            }
            ValueRef::Integer(_) => Err(InvalidFunctionParameterType(0, Type::Integer))?,
            ValueRef::Real(_) => Err(InvalidFunctionParameterType(0, Type::Real))?,
        }
    }
    Ok(if has_vals {
        Some(digest.finalize().to_vec())
    } else {
        None
    })
}

/// Convert a `Result<Option<Vec<u8>>>` to a `Result<Option<String>>` in upper case, same as SQLite `hex()` function.
#[cfg(feature = "hex")]
pub fn to_hex(value: Result<Option<Vec<u8>>>) -> Result<Option<String>> {
    value.map(|v| v.map(|v| v.encode_hex_upper()))
}
