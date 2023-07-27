use rusqlite::functions::FunctionFlags;
use rusqlite::types::{Type, ValueRef};
use rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};
use rusqlite::{Connection, Result};
use sha1::{Digest, Sha1};

pub fn create_fn_sha1(conn: &Connection) -> Result<()> {
    let flags = FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC;
    conn.create_scalar_function("sha1", 1, flags, |ctx| {
        if ctx.len() != 1 {
            return Err(InvalidParameterCount(ctx.len(), 1));
        }
        match ctx.get_raw(0) {
            ValueRef::Text(v) | ValueRef::Blob(v) => {
                let mut hasher = Sha1::default();
                hasher.update(v);
                Ok(Some(hasher.finalize().to_vec()))
            }
            ValueRef::Null => Ok(None),
            ValueRef::Integer(_) => Err(InvalidFunctionParameterType(0, Type::Integer)),
            ValueRef::Real(_) => Err(InvalidFunctionParameterType(0, Type::Integer)),
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sha1() -> Result<()> {
        let db = Connection::open_in_memory()?;
        let c = &db;
        create_fn_sha1(c)?;

        tst_hex(c, "NULL", None)?;
        tst_hex(c, "''", Some("da39a3ee5e6b4b0d3255bfef95601890afd80709"))?;
        tst_hex(c, "'a'", Some("86f7e437faa5a7fce15d1ddcb9eaeaea377667b8"))?;
        tst_hex(
            c,
            "'123456789'",
            Some("f7c3bc1d808e04732adf679965ccc34ca7ae3441"),
        )?;
        tst_hex(c, "x''", Some("da39a3ee5e6b4b0d3255bfef95601890afd80709"))?;
        tst_hex(c, "x'00'", Some("5ba93c9db0cff93f52b521d7420e43f6eda2784f"))?;
        tst_hex(
            c,
            "x'0123456789abcdef'",
            Some("0ca2eadb529ac2e63abf9b4ae3df8ee121f10547"),
        )?;

        assert!(tst_hex(c, "1", None).is_err());
        assert!(tst_hex(c, "0.42", None).is_err());

        tst_txt(
            c,
            "hex(sha1(''))",
            Some("DA39A3EE5E6B4B0D3255BFEF95601890AFD80709"),
        )?;
        tst_txt(
            c,
            "hex(sha1('a'))",
            Some("86F7E437FAA5A7FCE15D1DDCB9EAEAEA377667B8"),
        )?;
        tst_txt(
            c,
            "hex(sha1(x'00'))",
            Some("5BA93C9DB0CFF93F52B521D7420E43F6EDA2784F"),
        )?;

        Ok(())
    }

    fn tst_hex(db: &Connection, value: &str, expected: Option<&str>) -> Result<()> {
        let sql = format!("SELECT sha1({value})");
        let res: Option<Vec<_>> = db.query_row_and_then(&sql, [], |r| r.get(0))?;
        let res_str = res.map(|v| v.iter().map(|b| format!("{:02x}", b)).collect::<String>());
        assert_eq!(
            res_str,
            expected.map(|s| s.to_string()),
            "asserting hash for {value}"
        );
        Ok(())
    }

    fn tst_txt(db: &Connection, expr: &str, expected: Option<&str>) -> Result<()> {
        let sql = format!("SELECT {expr}");
        let res: Option<String> = db.query_row_and_then(&sql, [], |r| r.get(0))?;
        assert_eq!(
            res,
            expected.map(|s| s.to_string()),
            "asserting str for {expr}"
        );
        Ok(())
    }
}
