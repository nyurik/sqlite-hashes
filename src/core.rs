use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};

use digest::Digest;
use rusqlite::functions::{Aggregate, Context};

use crate::rusqlite::functions::FunctionFlags;
use crate::rusqlite::types::{Type, ValueRef};
use crate::rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};
use crate::rusqlite::{Connection, Result};

pub(crate) fn create_hash_fn<T: Digest + UnwindSafe + RefUnwindSafe + 'static>(
    conn: &Connection,
    fn_name: &str,
) -> Result<()> {
    conn.create_scalar_function(
        fn_name,
        -1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let param_count = ctx.len();
            if param_count == 0 {
                return Err(InvalidParameterCount(param_count, 1));
            }
            let mut digest = T::new();
            let mut has_vals = false;
            for idx in 0..param_count {
                match ctx.get_raw(idx) {
                    ValueRef::Text(v) | ValueRef::Blob(v) => {
                        digest.update(v);
                        has_vals = true;
                    }
                    ValueRef::Null => {}
                    ValueRef::Integer(_) => Err(InvalidFunctionParameterType(0, Type::Integer))?,
                    ValueRef::Real(_) => Err(InvalidFunctionParameterType(0, Type::Real))?,
                }
            }
            Ok(if has_vals {
                Some(digest.finalize().to_vec())
            } else {
                None
            })
        },
    )?;

    conn.create_aggregate_function(
        &format!("{fn_name}_concat"),
        -1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        HashAggregate::<T>(PhantomData),
    )
}

struct HashAggregate<T>(PhantomData<T>);

impl<T: Digest + UnwindSafe + RefUnwindSafe> Aggregate<(bool, T), Option<Vec<u8>>>
    for HashAggregate<T>
{
    fn init(&self, _: &mut Context<'_>) -> Result<(bool, T)> {
        // Keep track if any non-null values were added or not.
        // If there are, a non-null digest is returned.
        Ok((false, T::new()))
    }

    fn step(&self, ctx: &mut Context<'_>, digest: &mut (bool, T)) -> Result<()> {
        let param_count = ctx.len();
        if param_count == 0 {
            return Err(InvalidParameterCount(param_count, 1));
        }
        for idx in 0..param_count {
            match ctx.get_raw(idx) {
                ValueRef::Text(v) | ValueRef::Blob(v) => {
                    digest.0 = true;
                    digest.1.update(v);
                }
                ValueRef::Null => {}
                ValueRef::Integer(_) => Err(InvalidFunctionParameterType(0, Type::Integer))?,
                ValueRef::Real(_) => Err(InvalidFunctionParameterType(0, Type::Real))?,
            }
        }
        Ok(())
    }

    fn finalize(&self, _: &mut Context<'_>, digest: Option<(bool, T)>) -> Result<Option<Vec<u8>>> {
        Ok(digest.and_then(|(has_vals, v)| {
            if has_vals {
                Some(v.finalize().to_vec())
            } else {
                None
            }
        }))
    }
}
