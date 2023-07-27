use md5::Md5;

use crate::core::create_hash_fn;
use crate::rusqlite::{Connection, Result};

pub fn register_md5_function(conn: &Connection) -> Result<()> {
    create_hash_fn::<Md5>(conn, "md5")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::test::{hex, is_err, is_null, txt};

    #[test]
    fn test_md5() {
        let db = Connection::open_in_memory().unwrap();
        let c = &db;
        register_md5_function(c).unwrap();

        is_null(c, "md5(NULL)");
        is_err(c, "md5(1)");
        is_err(c, "md5(0.42)");
        is_err(c, "md5()");
        is_err(c, "md5('a', 'b')");

        hex(c, "md5('')", "d41d8cd98f00b204e9800998ecf8427e");
        hex(c, "md5('a')", "0cc175b9c0f1b6a831c399e269772661");
        hex(c, "md5('123456789')", "25f9e794323b453885f5181f1b624d0b");
        hex(c, "md5(x'')", "d41d8cd98f00b204e9800998ecf8427e");
        hex(c, "md5(x'00')", "93b885adfe0da089cdf634904fd59f71");
        hex(
            c,
            "md5(x'0123456789abcdef')",
            "a1cd1d1fc6491068d91007283ed84489",
        );

        txt(c, "hex(md5(''))", "D41D8CD98F00B204E9800998ECF8427E");
        txt(c, "hex(md5('a'))", "0CC175B9C0F1B6A831C399E269772661");
        txt(c, "hex(md5(x'00'))", "93B885ADFE0DA089CDF634904FD59F71");
    }
}
