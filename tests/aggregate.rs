#![cfg(feature = "aggregate")]
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

    /// The ordering here is un-documented and may change in the future.
    fn legacy_aggregate(&self, hash: &str) -> String {
        let sql = format!("SELECT {hash} FROM (SELECT b FROM tbl ORDER BY b)");
        self.str(&sql)
    }
}

#[test]
fn hash_concat() {
    let c = Conn::new();

    #[cfg(feature = "md5")]
    assert_snapshot!(c.legacy_aggregate("hex(md5_concat(b))"), @"D1AAF4767A3C10A473407A4E47B02DA6");
    #[cfg(feature = "sha1")]
    assert_snapshot!(c.legacy_aggregate("hex(sha1_concat(b))"), @"395E4981D467D1BD120DFB708ED4E3869C34BC04");
    #[cfg(feature = "sha224")]
    assert_snapshot!(c.legacy_aggregate("hex(sha224_concat(b))"), @"43A9BF6E729C8E813F4BAC4E5D9F6720338EF646FFF8B012D3D0AB36");
    #[cfg(feature = "sha256")]
    assert_snapshot!(c.legacy_aggregate("hex(sha256_concat(b))"), @"FB84A45F6DF7D1D17036F939F1CFEB87339FF5DBDF411222F3762DD76779A287");
    #[cfg(feature = "sha384")]
    assert_snapshot!(c.legacy_aggregate("hex(sha384_concat(b))"), @"4936373522EEB4FEA02B9F3F8B96E13ACD5E760FF765DEE10B74E7FFE1D3BFB33A93DC63B013DAB9F59FAEEC3205B5BE");
    #[cfg(feature = "sha512")]
    assert_snapshot!(c.legacy_aggregate("hex(sha512_concat(b))"), @"EEC013A2A7208C51FD20F975AAD231B2E21E7C1D9E228B2480E33C8F52AC482D82C8514CBEB6036D7FC76CB6262AE5780BBC628B0A6F2DF32E5255A21D4732F4");
}

#[test]
#[cfg(feature = "hex")]
fn hash_hex_concat() {
    let c = Conn::new();

    #[cfg(feature = "md5")]
    assert_snapshot!(c.legacy_aggregate("md5_hex_concat(b)"), @"D1AAF4767A3C10A473407A4E47B02DA6");
    #[cfg(feature = "sha1")]
    assert_snapshot!(c.legacy_aggregate("sha1_hex_concat(b)"), @"395E4981D467D1BD120DFB708ED4E3869C34BC04");
    #[cfg(feature = "sha224")]
    assert_snapshot!(c.legacy_aggregate("sha224_hex_concat(b)"), @"43A9BF6E729C8E813F4BAC4E5D9F6720338EF646FFF8B012D3D0AB36");
    #[cfg(feature = "sha256")]
    assert_snapshot!(c.legacy_aggregate("sha256_hex_concat(b)"), @"FB84A45F6DF7D1D17036F939F1CFEB87339FF5DBDF411222F3762DD76779A287");
    #[cfg(feature = "sha384")]
    assert_snapshot!(c.legacy_aggregate("sha384_hex_concat(b)"), @"4936373522EEB4FEA02B9F3F8B96E13ACD5E760FF765DEE10B74E7FFE1D3BFB33A93DC63B013DAB9F59FAEEC3205B5BE");
    #[cfg(feature = "sha512")]
    assert_snapshot!(c.legacy_aggregate("sha512_hex_concat(b)"), @"EEC013A2A7208C51FD20F975AAD231B2E21E7C1D9E228B2480E33C8F52AC482D82C8514CBEB6036D7FC76CB6262AE5780BBC628B0A6F2DF32E5255A21D4732F4");
}
