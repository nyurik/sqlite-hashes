#![allow(dead_code)]

use rusqlite::types::FromSql;
use rusqlite::{Connection, Error};

pub fn is_null(db: &Connection, expr: &str) {
    let sql = format!("SELECT {expr}");
    let res: Option<Vec<_>> = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
    assert_eq!(res, None, "asserting NULL result for {expr}");
}

pub fn is_err(db: &Connection, expr: &str) {
    let sql = format!("SELECT {expr}");
    let res: Result<Vec<_>, Error> = db.query_row_and_then(&sql, [], |r| r.get(0));
    assert!(res.is_err(), "asserting error result for {expr}");
}

pub fn txt(db: &Connection, expr: &str, expected: &str) {
    let sql = format!("SELECT {expr}");
    let res: String = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
    assert_eq!(res, expected, "asserting str for {expr}");
}

pub fn hex(db: &Connection, expr: &str, expected: &str) {
    let sql = format!("SELECT {expr}");
    let res: Vec<_> = db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap();
    let res_str = to_hex(&res);
    assert_eq!(res_str, expected, "asserting hash for {expr}");
}

pub fn seq(db: &Connection, expr: &str, iterations: usize, expected: Option<&str>) {
    let res = sequence::<Option<Vec<u8>>>(db, iterations, expr);
    let res_str = res.as_deref().map(to_hex);
    assert_eq!(res_str.as_deref(), expected, "asserting str for {expr}");

    let expr = format!("hex({expr})");
    let res = sequence::<String>(db, iterations, &expr);
    let expected = expected.map_or(String::new(), |s| s.to_uppercase());
    assert_eq!(res, expected, "asserting hex for {expr}");
}

fn sequence<T: FromSql>(db: &Connection, iterations: usize, expr: &str) -> T {
    // Modeled after https://stackoverflow.com/a/26241151/177275
    let sql = format!(
        "
WITH RECURSIVE
  seq(value) AS (
     SELECT 1
     UNION ALL
     SELECT value + 1 FROM seq
     LIMIT {iterations}
  )
SELECT {expr} FROM seq"
    );
    db.query_row_and_then(&sql, [], |r| r.get(0)).unwrap()
}

fn to_hex(res: &[u8]) -> String {
    res.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

struct Test {
    expr: &'static str,
    md5: &'static str,
    sha1: &'static str,
    sha256: &'static str,
    sha512: &'static str,
}
struct TestSeq {
    expr: &'static str,
    count: usize,
    md5: &'static str,
    sha1: &'static str,
    sha256: &'static str,
    sha512: &'static str,
}

fn get_connection() -> Connection {
    let db = Connection::open_in_memory().unwrap();
    let c = &db;
    #[cfg(feature = "md5")]
    sqlite_hashes::register_md5_function(c).unwrap();
    #[cfg(feature = "sha1")]
    sqlite_hashes::register_sha1_function(c).unwrap();
    #[cfg(feature = "sha256")]
    sqlite_hashes::register_sha256_function(c).unwrap();
    #[cfg(feature = "sha512")]
    sqlite_hashes::register_sha512_function(c).unwrap();
    db
}

fn hash(expr: &str, func: &'static str) -> String {
    let expr2 = expr.replace("HASH", func);
    assert_ne!(expr, expr2);
    expr2
}

#[test]
fn test_hex() {
    let c = &get_connection();
    for t in &[
            &Test {
                expr: "HASH('')",
                md5: "d41d8cd98f00b204e9800998ecf8427e",
                sha1: "da39a3ee5e6b4b0d3255bfef95601890afd80709",
                sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                sha512: "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
            },
            &Test {
                expr: "HASH('a')",
                md5: "0cc175b9c0f1b6a831c399e269772661",
                sha1: "86f7e437faa5a7fce15d1ddcb9eaeaea377667b8",
                sha256: "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb",
                sha512: "1f40fc92da241694750979ee6cf582f2d5d7d28e18335de05abc54d0560e0f5302860c652bf08d560252aa5e74210546f369fbbbce8c12cfc7957b2652fe9a75",
            },
            &Test {
                expr: "HASH('123456789')",
                md5: "25f9e794323b453885f5181f1b624d0b",
                sha1: "f7c3bc1d808e04732adf679965ccc34ca7ae3441",
                sha256: "15e2b0d3c33891ebb0f1ef609ec419420c20e320ce94c65fbc8c3312448eb225",
                sha512: "d9e6762dd1c8eaf6d61b3c6192fc408d4d6d5f1176d0c29169bc24e71c3f274ad27fcd5811b313d681f7e55ec02d73d499c95455b6b5bb503acf574fba8ffe85",
            },
            &Test {
                expr: "HASH(x'')",
                md5: "d41d8cd98f00b204e9800998ecf8427e",
                sha1: "da39a3ee5e6b4b0d3255bfef95601890afd80709",
                sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                sha512: "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
            },
            &Test {
                expr: "HASH(x'00')",
                md5: "93b885adfe0da089cdf634904fd59f71",
                sha1: "5ba93c9db0cff93f52b521d7420e43f6eda2784f",
                sha256: "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d",
                sha512: "b8244d028981d693af7b456af8efa4cad63d282e19ff14942c246e50d9351d22704a802a71c3580b6370de4ceb293c324a8423342557d4e5c38438f0e36910ee",
            },
            &Test {
                expr: "HASH(x'0123456789abcdef')",
                md5: "a1cd1d1fc6491068d91007283ed84489",
                sha1: "0ca2eadb529ac2e63abf9b4ae3df8ee121f10547",
                sha256: "55c53f5d490297900cefa825d0c8e8e9532ee8a118abe7d8570762cd38be9818",
                sha512: "650161856da7d9f818e6047cf6b2092bc7aa3767d3495cfbefe2b710ed684a43ba933ea8286ef67d975e64e0482e5ebe0701788989396545b6badb3b0a136f19",
            },
        ] {
            #[cfg(feature = "md5")]
            hex(c, &hash(t.expr, "md5"), t.md5);
            #[cfg(feature = "sha1")]
            hex(c, &hash(t.expr, "sha1"), t.sha1);
            #[cfg(feature = "sha256")]
            hex(c, &hash(t.expr, "sha256"), t.sha256);
            #[cfg(feature = "sha512")]
            hex(c, &hash(t.expr, "sha512"), t.sha512);

            #[cfg(feature = "md5")]
            hex(c, &hash(t.expr, "md5_concat"), t.md5);
            #[cfg(feature = "sha1")]
            hex(c, &hash(t.expr, "sha1_concat"), t.sha1);
            #[cfg(feature = "sha256")]
            hex(c, &hash(t.expr, "sha256_concat"), t.sha256);
            #[cfg(feature = "sha512")]
            hex(c, &hash(t.expr, "sha512_concat"), t.sha512);
        }
}

#[test]
fn test_txt() {
    let c = &get_connection();
    for t in &[
            &Test {
                expr: "hex(HASH(''))",
                md5: "D41D8CD98F00B204E9800998ECF8427E",
                sha1: "DA39A3EE5E6B4B0D3255BFEF95601890AFD80709",
                sha256: "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                sha512: "CF83E1357EEFB8BDF1542850D66D8007D620E4050B5715DC83F4A921D36CE9CE47D0D13C5D85F2B0FF8318D2877EEC2F63B931BD47417A81A538327AF927DA3E",
            },
            &Test {
                expr: "hex(HASH('a'))",
                md5: "0CC175B9C0F1B6A831C399E269772661",
                sha1: "86F7E437FAA5A7FCE15D1DDCB9EAEAEA377667B8",
                sha256: "CA978112CA1BBDCAFAC231B39A23DC4DA786EFF8147C4E72B9807785AFEE48BB",
                sha512: "1F40FC92DA241694750979EE6CF582F2D5D7D28E18335DE05ABC54D0560E0F5302860C652BF08D560252AA5E74210546F369FBBBCE8C12CFC7957B2652FE9A75",
            },
            &Test {
                expr: "hex(HASH(x'00'))",
                md5: "93B885ADFE0DA089CDF634904FD59F71",
                sha1: "5BA93C9DB0CFF93F52B521D7420E43F6EDA2784F",
                sha256: "6E340B9CFFB37A989CA544E6BB780A2C78901D3FB33738768511A30617AFA01D",
                sha512: "B8244D028981D693AF7B456AF8EFA4CAD63D282E19FF14942C246E50D9351D22704A802A71C3580B6370DE4CEB293C324A8423342557D4E5C38438F0E36910EE",
            },
        ] {
            #[cfg(feature = "md5")]
            txt(c, &hash(t.expr, "md5"), t.md5);
            #[cfg(feature = "sha1")]
            txt(c, &hash(t.expr, "sha1"), t.sha1);
            #[cfg(feature = "sha256")]
            txt(c, &hash(t.expr, "sha256"), t.sha256);
            #[cfg(feature = "sha512")]
            txt(c, &hash(t.expr, "sha512"), t.sha512);

            #[cfg(feature = "md5")]
            txt(c, &hash(t.expr, "md5_concat"), t.md5);
            #[cfg(feature = "sha1")]
            txt(c, &hash(t.expr, "sha1_concat"), t.sha1);
            #[cfg(feature = "sha256")]
            txt(c, &hash(t.expr, "sha256_concat"), t.sha256);
            #[cfg(feature = "sha512")]
            txt(c, &hash(t.expr, "sha512_concat"), t.sha512);
        }
}

#[test]
fn test_seq() {
    let c = &get_connection();
    for t in &[
            &TestSeq {
                expr: "HASH(cast(value as text))",
                count: 1,
                md5: "c4ca4238a0b923820dcc509a6f75849b",
                sha1: "356a192b7913b04c54574d18c28d46e6395428ab",
                sha256: "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
                sha512: "4dff4ea340f0a823f15d3f4f01ab62eae0e5da579ccb851f8db9dfe84c58b2b37b89903a740e1ee172da793a6e79d560e5f7f9bd058a12a280433ed6fa46510a",
            },
            &TestSeq {
                expr: "HASH(cast(value as text))",
                count: 1000,
                md5: "271da02691152c8d972cdd2080a718fe",
                sha1: "5039f17ceb356b83d50a5af4c9391e762cf9d822",
                sha256: "03f81a758eeeecf8a62453911d1c8c671f9ea46e90998eddd91afb06e22a3d01",
                sha512: "6c529391b053f969f48f11aee0ee8d5553f627ce960ca1049b1a481f627498bdf9e0a610c7fdfb979cc6307f16dbd139f5446117277bf9a1572607ec6d33d0ef",
            },
            &TestSeq {
                expr: "HASH(cast(value as text))",
                count: 0,
                md5: "None",
                sha1: "None",
                sha256: "None",
                sha512: "None",
            },
            &TestSeq {
                expr: "HASH(cast(value as blob))",
                count: 1,
                md5: "c4ca4238a0b923820dcc509a6f75849b",
                sha1: "356a192b7913b04c54574d18c28d46e6395428ab",
                sha256: "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
                sha512: "4dff4ea340f0a823f15d3f4f01ab62eae0e5da579ccb851f8db9dfe84c58b2b37b89903a740e1ee172da793a6e79d560e5f7f9bd058a12a280433ed6fa46510a",
            },
            &TestSeq {
                expr: "HASH(cast(value as blob))",
                count: 1000,
                md5: "271da02691152c8d972cdd2080a718fe",
                sha1: "5039f17ceb356b83d50a5af4c9391e762cf9d822",
                sha256: "03f81a758eeeecf8a62453911d1c8c671f9ea46e90998eddd91afb06e22a3d01",
                sha512: "6c529391b053f969f48f11aee0ee8d5553f627ce960ca1049b1a481f627498bdf9e0a610c7fdfb979cc6307f16dbd139f5446117277bf9a1572607ec6d33d0ef",
            },
            &TestSeq {
                expr: "HASH(cast(value as blob))",
                count: 0,
                md5: "None",
                sha1: "None",
                sha256: "None",
                sha512: "None",
            },
        ] {
            let cnv = |v| if v == "None" { None } else { Some(v) };
            #[cfg(feature = "md5")]
            seq(c, &hash(t.expr, "md5_concat"), t.count, cnv(t.md5));
            #[cfg(feature = "sha1")]
            seq(c, &hash(t.expr, "sha1_concat"), t.count, cnv(t.sha1));
            #[cfg(feature = "sha256")]
            seq(c, &hash(t.expr, "sha256_concat"), t.count, cnv(t.sha256));
            #[cfg(feature = "sha512")]
            seq(c, &hash(t.expr, "sha512_concat"), t.count, cnv(t.sha512));
        }
}

#[test]
#[cfg(any(
    feature = "md5",
    feature = "sha1",
    feature = "sha256",
    feature = "sha512"
))]
fn test_errors() {
    let c = &get_connection();
    for func in &[
        #[cfg(feature = "md5")]
        "md5",
        #[cfg(feature = "md5")]
        "md5_concat",
        #[cfg(feature = "sha1")]
        "sha1",
        #[cfg(feature = "sha1")]
        "sha1_concat",
        #[cfg(feature = "sha256")]
        "sha256",
        #[cfg(feature = "sha256")]
        "sha256_concat",
        #[cfg(feature = "sha512")]
        "sha512",
        #[cfg(feature = "sha512")]
        "sha512_concat",
    ] {
        is_null(c, &hash("HASH(NULL)", func));
        is_err(c, &hash("HASH(1)", func));
        is_err(c, &hash("HASH(0.42)", func));
        is_err(c, &hash("HASH()", func));
        is_err(c, &hash("HASH('a', 'b')", func));
    }
}
