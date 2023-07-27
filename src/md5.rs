use md5::{Digest, Md5};
use rusqlite::functions::FunctionFlags;
use rusqlite::types::{Type, ValueRef};
use rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};
use rusqlite::{Connection, Result};

pub fn create_fn_md5(conn: &Connection) -> Result<()> {
    let flags = FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC;
    conn.create_scalar_function("md5", 1, flags, |ctx| {
        if ctx.len() != 1 {
            return Err(InvalidParameterCount(ctx.len(), 1));
        }
        match ctx.get_raw(0) {
            ValueRef::Text(v) | ValueRef::Blob(v) => {
                let mut hasher = Md5::default();
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
    fn test_md5() -> Result<()> {
        let db = Connection::open_in_memory()?;
        let c = &db;
        create_fn_md5(c)?;

        tst_hex(c, "NULL", None)?;
        tst_hex(c, "''", Some("d41d8cd98f00b204e9800998ecf8427e"))?;
        tst_hex(c, "'a'", Some("0cc175b9c0f1b6a831c399e269772661"))?;
        tst_hex(c, "'123456789'", Some("25f9e794323b453885f5181f1b624d0b"))?;
        tst_hex(c, "x''", Some("d41d8cd98f00b204e9800998ecf8427e"))?;
        tst_hex(c, "x'00'", Some("93b885adfe0da089cdf634904fd59f71"))?;
        tst_hex(
            c,
            "x'0123456789abcdef'",
            Some("a1cd1d1fc6491068d91007283ed84489"),
        )?;

        tst_txt(c, "hex(md5(''))", Some("D41D8CD98F00B204E9800998ECF8427E"))?;
        tst_txt(c, "hex(md5('a'))", Some("0CC175B9C0F1B6A831C399E269772661"))?;
        tst_txt(
            c,
            "hex(md5(x'00'))",
            Some("93B885ADFE0DA089CDF634904FD59F71"),
        )?;

        assert!(tst_hex(c, "1", None).is_err());
        assert!(tst_hex(c, "0.42", None).is_err());
        assert!(tst_hex(c, "", None).is_err());
        assert!(tst_hex(c, "'a', 'b'", None).is_err());

        Ok(())
    }

    fn tst_hex(db: &Connection, value: &str, expected: Option<&str>) -> Result<()> {
        let sql = format!("SELECT md5({value})");
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
