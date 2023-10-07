#[cfg(feature = "trace")]
use std::borrow::Cow;
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};

use digest::Digest;
#[cfg(feature = "trace")]
use hex::ToHex as _;
#[cfg(feature = "trace")]
use log::trace;
#[cfg(feature = "window")]
use rusqlite::functions::WindowAggregate;
use rusqlite::functions::{Aggregate, Context, FunctionFlags};
#[cfg(feature = "window")]
use rusqlite::Error::UserFunctionError;
use rusqlite::{Connection, ToSql};

use crate::rusqlite::types::{Type, ValueRef};
use crate::rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};
use crate::rusqlite::Result;

#[cfg(not(feature = "trace"))]
macro_rules! trace {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "window"))]
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

#[cfg(feature = "window")]
pub fn create_agg_function<A, W, T>(conn: &Connection, fn_name: &str, aggr: W) -> Result<()>
where
    A: RefUnwindSafe + UnwindSafe,
    W: WindowAggregate<A, T> + 'static,
    T: ToSql,
{
    trace!("Registering window function {fn_name}");
    conn.create_window_function(
        fn_name,
        -1,
        FunctionFlags::SQLITE_UTF8
            | FunctionFlags::SQLITE_DETERMINISTIC
            | FunctionFlags::SQLITE_DIRECTONLY,
        aggr,
    )
}

#[derive(Debug)]
pub struct AggState<T>(Option<T>);

impl<T: Digest + Clone + Clone> AggState<T> {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn add_value(&mut self, value: &[u8]) {
        self.0.get_or_insert_with(T::new).update(value);
    }

    #[cfg(feature = "window")]
    pub fn calc(&self) -> Option<Vec<u8>> {
        self.0.as_ref().map(|v| v.clone().finalize().to_vec())
    }

    pub fn finalize(self) -> Option<Vec<u8>> {
        self.0.map(|v| v.finalize().to_vec())
    }
}

pub struct AggType<T> {
    #[cfg(any(feature = "window", feature = "trace"))]
    fn_name: String,
    digest_type: PhantomData<T>,
}

impl<T: Digest + Clone + UnwindSafe + RefUnwindSafe> AggType<T> {
    #[cfg(any(feature = "window", feature = "trace"))]
    pub fn new(fn_name: String) -> Self {
        Self {
            fn_name: fn_name.to_ascii_uppercase(),
            digest_type: PhantomData,
        }
    }
    #[cfg(not(any(feature = "window", feature = "trace")))]
    pub fn new() -> Self {
        Self {
            digest_type: PhantomData,
        }
    }
}

impl<T: Digest + Clone + UnwindSafe + RefUnwindSafe> Aggregate<AggState<T>, Option<Vec<u8>>>
    for AggType<T>
{
    fn init(&self, _: &mut Context<'_>) -> Result<AggState<T>> {
        trace!("{}: Aggregate::init", self.fn_name);
        // Keep track if any non-null values were added or not.
        // If there are, a non-null digest is returned.
        Ok(AggState::new())
    }

    fn step(&self, ctx: &mut Context<'_>, agg: &mut AggState<T>) -> Result<()> {
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
                            Ok(v) => std::borrow::Cow::from(v),
                            Err(_) => Cow::from(val.encode_hex_upper::<String>()),
                        }
                    );
                    agg.add_value(val);
                }
                ValueRef::Null => {
                    trace!("{}: arg{idx} -> ignoring step(NULL)", self.fn_name);
                }
                ValueRef::Integer(_) => Err(InvalidFunctionParameterType(idx, Type::Integer))?,
                ValueRef::Real(_) => Err(InvalidFunctionParameterType(idx, Type::Real))?,
            }
        }
        Ok(())
    }

    fn finalize(&self, _: &mut Context<'_>, agg: Option<AggState<T>>) -> Result<Option<Vec<u8>>> {
        trace!("{}: Aggregate::finalize", self.fn_name);
        Ok(agg.and_then(|v| v.finalize()))
    }
}

#[cfg(feature = "window")]
impl<T: Digest + Clone + UnwindSafe + RefUnwindSafe> WindowAggregate<AggState<T>, Option<Vec<u8>>>
    for AggType<T>
{
    fn value(&self, agg: Option<&AggState<T>>) -> Result<Option<Vec<u8>>> {
        trace!("{}: WindowAggregate::value", self.fn_name);
        Ok(agg.and_then(|v| v.calc()))
    }

    fn inverse(&self, _: &mut Context<'_>, _: &mut AggState<T>) -> Result<()> {
        trace!("{}: WindowAggregate::inverse", self.fn_name);
        Err(UserFunctionError(
            format!(
                "Function {}() does not support Window size.  Use ordering only.",
                self.fn_name
            )
            .into(),
        ))
    }
}

#[cfg(feature = "hex")]
pub struct AggHexType<T>(AggType<T>);

#[cfg(feature = "hex")]
impl<T: Digest + Clone + UnwindSafe + RefUnwindSafe> AggHexType<T> {
    #[cfg(any(feature = "window", feature = "trace"))]
    pub fn new(fn_name: String) -> Self {
        Self(AggType::new(fn_name))
    }
    #[cfg(not(any(feature = "window", feature = "trace")))]
    pub fn new() -> Self {
        Self(AggType::new())
    }
}

#[cfg(feature = "hex")]
impl<T: Digest + Clone + UnwindSafe + RefUnwindSafe> Aggregate<AggState<T>, Option<String>>
    for AggHexType<T>
{
    fn init(&self, ctx: &mut Context<'_>) -> Result<AggState<T>> {
        self.0.init(ctx)
    }

    fn step(&self, ctx: &mut Context<'_>, acc: &mut AggState<T>) -> Result<()> {
        self.0.step(ctx, acc)
    }

    fn finalize(&self, ctx: &mut Context<'_>, acc: Option<AggState<T>>) -> Result<Option<String>> {
        crate::scalar::to_hex(self.0.finalize(ctx, acc))
    }
}

#[cfg(all(feature = "window", feature = "hex"))]
impl<T: Digest + Clone + UnwindSafe + RefUnwindSafe> WindowAggregate<AggState<T>, Option<String>>
    for AggHexType<T>
{
    fn value(&self, agg: Option<&AggState<T>>) -> Result<Option<String>> {
        crate::scalar::to_hex(self.0.value(agg))
    }

    fn inverse(&self, ctx: &mut Context<'_>, agg: &mut AggState<T>) -> Result<()> {
        self.0.inverse(ctx, agg)
    }
}
