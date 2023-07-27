use digest::Digest;

use crate::rusqlite::functions::FunctionFlags;
use crate::rusqlite::types::{Type, ValueRef};
use crate::rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};
use crate::rusqlite::{Connection, Result};

pub(crate) fn create_hash_fn<T: Digest>(conn: &Connection, fn_name: &str) -> Result<()> {
    let flags = FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC;
    conn.create_scalar_function(fn_name, 1, flags, |ctx| {
        if ctx.len() != 1 {
            return Err(InvalidParameterCount(ctx.len(), 1));
        }
        match ctx.get_raw(0) {
            ValueRef::Text(v) | ValueRef::Blob(v) => {
                let mut digest = T::new();
                digest.update(v);
                Ok(Some(digest.finalize().to_vec()))
            }
            ValueRef::Null => Ok(None),
            ValueRef::Integer(_) => Err(InvalidFunctionParameterType(0, Type::Integer)),
            ValueRef::Real(_) => Err(InvalidFunctionParameterType(0, Type::Real)),
        }
    })
}

#[cfg(test)]
pub mod test {
    use crate::rusqlite::{Connection, Result};

    pub fn is_null(db: &Connection, expr: &str) {
        let sql = format!("SELECT {expr}");
        let res: Option<Vec<_>> = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
        assert_eq!(res, None, "asserting NULL result for {expr}");
    }

    pub fn is_err(db: &Connection, expr: &str) {
        let sql = format!("SELECT {expr}");
        let res: Result<Vec<_>> = db.query_row_and_then(&sql, [], |r| r.get(0));
        assert!(res.is_err(), "asserting error result for {expr}");
    }

    pub fn hex(db: &Connection, expr: &str, expected: &str) {
        let sql = format!("SELECT {expr}");
        let res: Vec<_> = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
        let res_str = res.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        assert_eq!(res_str, expected, "asserting hash for {expr}");
    }

    pub fn txt(db: &Connection, expr: &str, expected: &str) {
        let sql = format!("SELECT {expr}");
        let res: String = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
        assert_eq!(res, expected, "asserting str for {expr}");
    }
}
