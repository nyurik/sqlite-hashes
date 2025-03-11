#![cfg(feature = "aggregate")]

#[cfg(feature = "trace")]
use std::borrow::Cow;
use std::panic::{RefUnwindSafe, UnwindSafe};

use digest::Digest;
#[cfg(feature = "trace")]
use hex::ToHex as _;
#[cfg(feature = "trace")]
use log::trace;
use rusqlite::functions::{Aggregate, Context, FunctionFlags};
use rusqlite::{Connection, ToSql};

use crate::rusqlite::types::{Type, ValueRef};
use crate::rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};
use crate::rusqlite::Result;
use crate::state::HashState;

#[cfg(not(feature = "trace"))]
macro_rules! trace {
    ($($arg:tt)*) => {};
}

pub fn create_agg_function<A, D, T>(conn: &Connection, fn_name: &str, aggr: D) -> Result<()>
where
    A: RefUnwindSafe + UnwindSafe,
    D: Aggregate<A, T> + 'static,
    T: ToSql,
{
    trace!("Registering aggregate function {fn_name}");
    conn.create_aggregate_function(
        fn_name,
        -1,
        FunctionFlags::SQLITE_UTF8
            | FunctionFlags::SQLITE_DETERMINISTIC
            | FunctionFlags::SQLITE_DIRECTONLY,
        aggr,
    )
}

pub struct AggType<D, R> {
    #[cfg(feature = "trace")]
    fn_name: String,
    to_final: fn(HashState<D>) -> Option<R>,
}

impl<D: Digest + Clone + UnwindSafe + RefUnwindSafe, R> AggType<D, R> {
    pub fn new(
        #[cfg(feature = "trace")] fn_name: &str,
        to_final: fn(HashState<D>) -> Option<R>,
    ) -> Self {
        Self {
            #[cfg(feature = "trace")]
            fn_name: fn_name.to_ascii_uppercase(),
            to_final,
        }
    }
}

impl<T: Digest + Clone + UnwindSafe + RefUnwindSafe, R: ToSql> Aggregate<HashState<T>, Option<R>>
    for AggType<T, R>
{
    fn init(&self, _: &mut Context<'_>) -> Result<HashState<T>> {
        trace!("{}: Aggregate::init", self.fn_name);
        // Keep track if any non-null values were added or not.
        // If there are, a non-null digest is returned.
        Ok(HashState::default())
    }

    fn step(&self, ctx: &mut Context<'_>, agg: &mut HashState<T>) -> Result<()> {
        let param_count = ctx.len();
        if param_count == 0 {
            return Err(InvalidParameterCount(param_count, 1));
        }
        for idx in 0..param_count {
            match ctx.get_raw(idx) {
                ValueRef::Blob(val) => {
                    trace!("{}: arg{idx} -> step(blob {val:?})", self.fn_name);
                    agg.add_value(val);
                }
                ValueRef::Text(val) => {
                    trace!(
                        "{}: arg{idx} -> step(text {:?})",
                        self.fn_name,
                        match std::str::from_utf8(val) {
                            Ok(v) => Cow::from(v),
                            Err(_) => Cow::from(val.encode_hex_upper::<String>()),
                        }
                    );
                    agg.add_value(val);
                }
                ValueRef::Null => {
                    trace!("{}: arg{idx} -> ignoring step(NULL)", self.fn_name);
                    agg.add_null();
                }
                ValueRef::Integer(_) => Err(InvalidFunctionParameterType(idx, Type::Integer))?,
                ValueRef::Real(_) => Err(InvalidFunctionParameterType(idx, Type::Real))?,
            }
        }
        Ok(())
    }

    fn finalize(&self, _: &mut Context<'_>, agg: Option<HashState<T>>) -> Result<Option<R>> {
        trace!("{}: Aggregate::finalize", self.fn_name);
        match agg {
            Some(agg) => Ok((self.to_final)(agg)),
            None => Ok(None),
        }
    }
}
