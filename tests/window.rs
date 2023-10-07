#![cfg(feature = "window")]
#![allow(dead_code)]

use insta::assert_snapshot;
use rusqlite::types::FromSql;
use rusqlite::Connection;

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

struct Conn(Connection);

impl Conn {
    fn new() -> Self {
        let db = Connection::open_in_memory().unwrap();
        sqlite_hashes::register_hash_functions(&db).unwrap();
        db.execute_batch(
            "
CREATE TABLE tbl(a INTEGER PRIMARY KEY, b TEXT);
INSERT INTO tbl VALUES (1, 'bbb'), (2, 'ccc'), (3, 'aaa');
",
        )
        .unwrap();
        Self(db)
    }

    fn sql<T: FromSql>(&self, query: &str) -> T {
        self.0.query_row_and_then(query, [], |r| r.get(0)).unwrap()
    }

    fn str(&self, query: &str) -> String {
        self.sql(query)
    }

    #[cfg(feature = "hex")]
    fn window(&self, hex_hash: &str) -> String {
        let sql = format!("SELECT {hex_hash}(b) OVER (ORDER BY b ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) FROM tbl LIMIT 1");
        self.str(&sql)
    }

    fn window_hex(&self, hash: &str) -> String {
        let sql = format!("SELECT hex((SELECT {hash}(b) OVER (ORDER BY b ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) FROM tbl LIMIT 1))");
        self.str(&sql)
    }
}

#[test]
fn window() {
    let c = Conn::new();

    #[cfg(feature = "md5")]
    assert_snapshot!(c.window_hex("md5_concat"), @"D1AAF4767A3C10A473407A4E47B02DA6");
    #[cfg(feature = "sha1")]
    assert_snapshot!(c.window_hex("sha1_concat"), @"395E4981D467D1BD120DFB708ED4E3869C34BC04");
    #[cfg(feature = "sha224")]
    assert_snapshot!(c.window_hex("sha224_concat"), @"43A9BF6E729C8E813F4BAC4E5D9F6720338EF646FFF8B012D3D0AB36");
    #[cfg(feature = "sha256")]
    assert_snapshot!(c.window_hex("sha256_concat"), @"FB84A45F6DF7D1D17036F939F1CFEB87339FF5DBDF411222F3762DD76779A287");
    #[cfg(feature = "sha384")]
    assert_snapshot!(c.window_hex("sha384_concat"), @"4936373522EEB4FEA02B9F3F8B96E13ACD5E760FF765DEE10B74E7FFE1D3BFB33A93DC63B013DAB9F59FAEEC3205B5BE");
    #[cfg(feature = "sha512")]
    assert_snapshot!(c.window_hex("sha512_concat"), @"EEC013A2A7208C51FD20F975AAD231B2E21E7C1D9E228B2480E33C8F52AC482D82C8514CBEB6036D7FC76CB6262AE5780BBC628B0A6F2DF32E5255A21D4732F4");
}

#[test]
#[cfg(feature = "hex")]
fn window_hex() {
    let c = Conn::new();

    #[cfg(all(feature = "md5", feature = "hex"))]
    assert_snapshot!(c.window("md5_hex_concat"), @"D1AAF4767A3C10A473407A4E47B02DA6");
    #[cfg(all(feature = "sha1", feature = "hex"))]
    assert_snapshot!(c.window("sha1_hex_concat"), @"395E4981D467D1BD120DFB708ED4E3869C34BC04");
    #[cfg(all(feature = "sha224", feature = "hex"))]
    assert_snapshot!(c.window("sha224_hex_concat"), @"43A9BF6E729C8E813F4BAC4E5D9F6720338EF646FFF8B012D3D0AB36");
    #[cfg(all(feature = "sha256", feature = "hex"))]
    assert_snapshot!(c.window("sha256_hex_concat"), @"FB84A45F6DF7D1D17036F939F1CFEB87339FF5DBDF411222F3762DD76779A287");
    #[cfg(all(feature = "sha384", feature = "hex"))]
    assert_snapshot!(c.window("sha384_hex_concat"), @"4936373522EEB4FEA02B9F3F8B96E13ACD5E760FF765DEE10B74E7FFE1D3BFB33A93DC63B013DAB9F59FAEEC3205B5BE");
    #[cfg(all(feature = "sha512", feature = "hex"))]
    assert_snapshot!(c.window("sha512_hex_concat"), @"EEC013A2A7208C51FD20F975AAD231B2E21E7C1D9E228B2480E33C8F52AC482D82C8514CBEB6036D7FC76CB6262AE5780BBC628B0A6F2DF32E5255A21D4732F4");
}
