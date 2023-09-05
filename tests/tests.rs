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
    let res: Result<Option<Vec<_>>, Error> = db.query_row_and_then(&sql, [], |r| r.get(0));
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
    sha224: &'static str,
    sha256: &'static str,
    sha384: &'static str,
    sha512: &'static str,
}
struct TestSeq {
    expr: &'static str,
    count: usize,
    md5: &'static str,
    sha1: &'static str,
    sha224: &'static str,
    sha256: &'static str,
    sha384: &'static str,
    sha512: &'static str,
}

fn get_connection() -> Connection {
    let db = Connection::open_in_memory().unwrap();
    sqlite_hashes::register_hash_functions(&db).unwrap();
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
                sha224: "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f",
                sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                sha384: "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b",
                sha512: "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
            },
            &Test {
                expr: "HASH('a')",
                md5: "0cc175b9c0f1b6a831c399e269772661",
                sha1: "86f7e437faa5a7fce15d1ddcb9eaeaea377667b8",
                sha224: "abd37534c7d9a2efb9465de931cd7055ffdb8879563ae98078d6d6d5",
                sha256: "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb",
                sha384: "54a59b9f22b0b80880d8427e548b7c23abd873486e1f035dce9cd697e85175033caa88e6d57bc35efae0b5afd3145f31",
                sha512: "1f40fc92da241694750979ee6cf582f2d5d7d28e18335de05abc54d0560e0f5302860c652bf08d560252aa5e74210546f369fbbbce8c12cfc7957b2652fe9a75",
            },
            &Test {
                expr: "HASH('123456789')",
                md5: "25f9e794323b453885f5181f1b624d0b",
                sha1: "f7c3bc1d808e04732adf679965ccc34ca7ae3441",
                sha224: "9b3e61bf29f17c75572fae2e86e17809a4513d07c8a18152acf34521",
                sha256: "15e2b0d3c33891ebb0f1ef609ec419420c20e320ce94c65fbc8c3312448eb225",
                sha384: "eb455d56d2c1a69de64e832011f3393d45f3fa31d6842f21af92d2fe469c499da5e3179847334a18479c8d1dedea1be3",
                sha512: "d9e6762dd1c8eaf6d61b3c6192fc408d4d6d5f1176d0c29169bc24e71c3f274ad27fcd5811b313d681f7e55ec02d73d499c95455b6b5bb503acf574fba8ffe85",
            },
            &Test {
                expr: "HASH(x'')",
                md5: "d41d8cd98f00b204e9800998ecf8427e",
                sha1: "da39a3ee5e6b4b0d3255bfef95601890afd80709",
                sha224: "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f",
                sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                sha384: "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b",
                sha512: "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
            },
            &Test {
                expr: "HASH(x'00')",
                md5: "93b885adfe0da089cdf634904fd59f71",
                sha1: "5ba93c9db0cff93f52b521d7420e43f6eda2784f",
                sha224: "fff9292b4201617bdc4d3053fce02734166a683d7d858a7f5f59b073",
                sha256: "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d",
                sha384: "bec021b4f368e3069134e012c2b4307083d3a9bdd206e24e5f0d86e13d6636655933ec2b413465966817a9c208a11717",
                sha512: "b8244d028981d693af7b456af8efa4cad63d282e19ff14942c246e50d9351d22704a802a71c3580b6370de4ceb293c324a8423342557d4e5c38438f0e36910ee",
            },
            &Test {
                expr: "HASH(x'0123456789abcdef')",
                md5: "a1cd1d1fc6491068d91007283ed84489",
                sha1: "0ca2eadb529ac2e63abf9b4ae3df8ee121f10547",
                sha224: "a4aec60feebae6312a18424e7d758e5b7f6f0c0b0854ecc365adbbce",
                sha256: "55c53f5d490297900cefa825d0c8e8e9532ee8a118abe7d8570762cd38be9818",
                sha384: "1ab07bdd5da4ed52bb7105d879671f88bf85a822afe6f21323b6cd89d7a7831ccde33c23ad0d014b6bb41a380d252af3",
                sha512: "650161856da7d9f818e6047cf6b2092bc7aa3767d3495cfbefe2b710ed684a43ba933ea8286ef67d975e64e0482e5ebe0701788989396545b6badb3b0a136f19",
            },
            &Test {
                expr: "HASH('', 'a', '123456789', x'', x'00', x'0123456789abcdef')",
                md5: "357484b18bee5b6190cebd93c5a89a4e",
                sha1: "af066c53d4e9e3d43690e46b228700e541d75187",
                sha224: "f4b3f8a5fea53a82123b28cb47b80c78ce720f071b50c319ddea0119",
                sha256: "a6482bb4fa2bcf19b4e7e06dd6e48651269020f35679f3e4d43af0dc6bb815a9",
                sha384: "2f45afdf7a593adf7db439bd78aaa9dde160b93416f9a2d7618213a1cca2f169673bb8f409d3ca07950f18a4e8718ace",
                sha512: "8aa00accb3bb3c17e44860b91f406d6bbca6c0aa8fc7e1192e3cb537a57019565e3267f7be9530adb6af5a50f67a1d1b17ab3fa24113b7caf6da316ac0a5b4e3",
            },
            &Test {
                expr: "HASH(NULL, 'a', NULL, '123456789', x'', x'00', NULL, x'0123456789abcdef', NULL)",
                md5: "357484b18bee5b6190cebd93c5a89a4e",
                sha1: "af066c53d4e9e3d43690e46b228700e541d75187",
                sha224: "f4b3f8a5fea53a82123b28cb47b80c78ce720f071b50c319ddea0119",
                sha256: "a6482bb4fa2bcf19b4e7e06dd6e48651269020f35679f3e4d43af0dc6bb815a9",
                sha384: "2f45afdf7a593adf7db439bd78aaa9dde160b93416f9a2d7618213a1cca2f169673bb8f409d3ca07950f18a4e8718ace",
                sha512: "8aa00accb3bb3c17e44860b91f406d6bbca6c0aa8fc7e1192e3cb537a57019565e3267f7be9530adb6af5a50f67a1d1b17ab3fa24113b7caf6da316ac0a5b4e3",
            },
        ] {
            #[cfg(feature = "md5")]
            hex(c, &hash(t.expr, "md5"), t.md5);
            #[cfg(feature = "sha1")]
            hex(c, &hash(t.expr, "sha1"), t.sha1);
            #[cfg(feature = "sha224")]
            hex(c, &hash(t.expr, "sha224"), t.sha224);
            #[cfg(feature = "sha256")]
            hex(c, &hash(t.expr, "sha256"), t.sha256);
            #[cfg(feature = "sha384")]
            hex(c, &hash(t.expr, "sha384"), t.sha384);
            #[cfg(feature = "sha512")]
            hex(c, &hash(t.expr, "sha512"), t.sha512);

            #[cfg(feature = "md5")]
            hex(c, &hash(t.expr, "md5_concat"), t.md5);
            #[cfg(feature = "sha1")]
            hex(c, &hash(t.expr, "sha1_concat"), t.sha1);
            #[cfg(feature = "sha224")]
            hex(c, &hash(t.expr, "sha224_concat"), t.sha224);
            #[cfg(feature = "sha256")]
            hex(c, &hash(t.expr, "sha256_concat"), t.sha256);
            #[cfg(feature = "sha384")]
            hex(c, &hash(t.expr, "sha384_concat"), t.sha384);
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
                sha224: "D14A028C2A3A2BC9476102BB288234C415A2B01F828EA62AC5B3E42F",
                sha256: "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                sha384: "38B060A751AC96384CD9327EB1B1E36A21FDB71114BE07434C0CC7BF63F6E1DA274EDEBFE76F65FBD51AD2F14898B95B",
                sha512: "CF83E1357EEFB8BDF1542850D66D8007D620E4050B5715DC83F4A921D36CE9CE47D0D13C5D85F2B0FF8318D2877EEC2F63B931BD47417A81A538327AF927DA3E",
            },
            &Test {
                expr: "hex(HASH('a'))",
                md5: "0CC175B9C0F1B6A831C399E269772661",
                sha1: "86F7E437FAA5A7FCE15D1DDCB9EAEAEA377667B8",
                sha224: "ABD37534C7D9A2EFB9465DE931CD7055FFDB8879563AE98078D6D6D5",
                sha256: "CA978112CA1BBDCAFAC231B39A23DC4DA786EFF8147C4E72B9807785AFEE48BB",
                sha384: "54A59B9F22B0B80880D8427E548B7C23ABD873486E1F035DCE9CD697E85175033CAA88E6D57BC35EFAE0B5AFD3145F31",
                sha512: "1F40FC92DA241694750979EE6CF582F2D5D7D28E18335DE05ABC54D0560E0F5302860C652BF08D560252AA5E74210546F369FBBBCE8C12CFC7957B2652FE9A75",
            },
            &Test {
                expr: "hex(HASH(x'00'))",
                md5: "93B885ADFE0DA089CDF634904FD59F71",
                sha1: "5BA93C9DB0CFF93F52B521D7420E43F6EDA2784F",
                sha224: "FFF9292B4201617BDC4D3053FCE02734166A683D7D858A7F5F59B073",
                sha256: "6E340B9CFFB37A989CA544E6BB780A2C78901D3FB33738768511A30617AFA01D",
                sha384: "BEC021B4F368E3069134E012C2B4307083D3A9BDD206E24E5F0D86E13D6636655933EC2B413465966817A9C208A11717",
                sha512: "B8244D028981D693AF7B456AF8EFA4CAD63D282E19FF14942C246E50D9351D22704A802A71C3580B6370DE4CEB293C324A8423342557D4E5C38438F0E36910EE",
            },
        ] {
            #[cfg(feature = "md5")]
            txt(c, &hash(t.expr, "md5"), t.md5);
            #[cfg(feature = "sha1")]
            txt(c, &hash(t.expr, "sha1"), t.sha1);
            #[cfg(feature = "sha224")]
            txt(c, &hash(t.expr, "sha224"), t.sha224);
            #[cfg(feature = "sha256")]
            txt(c, &hash(t.expr, "sha256"), t.sha256);
            #[cfg(feature = "sha384")]
            txt(c, &hash(t.expr, "sha384"), t.sha384);
            #[cfg(feature = "sha512")]
            txt(c, &hash(t.expr, "sha512"), t.sha512);

            #[cfg(feature = "md5")]
            txt(c, &hash(t.expr, "md5_concat"), t.md5);
            #[cfg(feature = "sha1")]
            txt(c, &hash(t.expr, "sha1_concat"), t.sha1);
            #[cfg(feature = "sha224")]
            txt(c, &hash(t.expr, "sha224_concat"), t.sha224);
            #[cfg(feature = "sha256")]
            txt(c, &hash(t.expr, "sha256_concat"), t.sha256);
            #[cfg(feature = "sha384")]
            txt(c, &hash(t.expr, "sha384_concat"), t.sha384);
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
                sha224: "e25388fde8290dc286a6164fa2d97e551b53498dcbf7bc378eb1f178",
                sha256: "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
                sha384: "47f05d367b0c32e438fb63e6cf4a5f35c2aa2f90dc7543f8a41a0f95ce8a40a313ab5cf36134a2068c4c969cb50db776",
                sha512: "4dff4ea340f0a823f15d3f4f01ab62eae0e5da579ccb851f8db9dfe84c58b2b37b89903a740e1ee172da793a6e79d560e5f7f9bd058a12a280433ed6fa46510a",
            },
            &TestSeq {
                expr: "HASH(cast(value as text))",
                count: 1000,
                md5: "271da02691152c8d972cdd2080a718fe",
                sha1: "5039f17ceb356b83d50a5af4c9391e762cf9d822",
                sha224: "b18b990ab98bf6fc8b02d594645faf7d4a1a45b5846d6b713af00eed",
                sha256: "03f81a758eeeecf8a62453911d1c8c671f9ea46e90998eddd91afb06e22a3d01",
                sha384: "8848c1b74b0f6967042ab28c9bd27cdaf92349b49fd78a88fa82e1705de031f7ade7488a0fdb6a2c60ae5cb7587be49d",
                sha512: "6c529391b053f969f48f11aee0ee8d5553f627ce960ca1049b1a481f627498bdf9e0a610c7fdfb979cc6307f16dbd139f5446117277bf9a1572607ec6d33d0ef",
            },
            &TestSeq {
                expr: "HASH(cast(value as text))",
                count: 0,
                md5: "None",
                sha1: "None",
                sha224: "None",
                sha256: "None",
                sha384: "None",
                sha512: "None",
            },
            &TestSeq {
                expr: "HASH(cast(value as blob))",
                count: 1,
                md5: "c4ca4238a0b923820dcc509a6f75849b",
                sha1: "356a192b7913b04c54574d18c28d46e6395428ab",
                sha224: "e25388fde8290dc286a6164fa2d97e551b53498dcbf7bc378eb1f178",
                sha256: "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",
                sha384: "47f05d367b0c32e438fb63e6cf4a5f35c2aa2f90dc7543f8a41a0f95ce8a40a313ab5cf36134a2068c4c969cb50db776",
                sha512: "4dff4ea340f0a823f15d3f4f01ab62eae0e5da579ccb851f8db9dfe84c58b2b37b89903a740e1ee172da793a6e79d560e5f7f9bd058a12a280433ed6fa46510a",
            },
            &TestSeq {
                expr: "HASH(cast(value as blob))",
                count: 1000,
                md5: "271da02691152c8d972cdd2080a718fe",
                sha1: "5039f17ceb356b83d50a5af4c9391e762cf9d822",
                sha224: "b18b990ab98bf6fc8b02d594645faf7d4a1a45b5846d6b713af00eed",
                sha256: "03f81a758eeeecf8a62453911d1c8c671f9ea46e90998eddd91afb06e22a3d01",
                sha384: "8848c1b74b0f6967042ab28c9bd27cdaf92349b49fd78a88fa82e1705de031f7ade7488a0fdb6a2c60ae5cb7587be49d",
                sha512: "6c529391b053f969f48f11aee0ee8d5553f627ce960ca1049b1a481f627498bdf9e0a610c7fdfb979cc6307f16dbd139f5446117277bf9a1572607ec6d33d0ef",
            },
            &TestSeq {
                expr: "HASH(cast(value as blob))",
                count: 0,
                md5: "None",
                sha1: "None",
                sha224: "None",
                sha256: "None",
                sha384: "None",
                sha512: "None",
            },
            &TestSeq {
                expr: "HASH(cast(value as text), cast((value+1) as blob))",
                count: 1,
                md5: "c20ad4d76fe97759aa27a0c99bff6710",
                sha1: "7b52009b64fd0a2a49e6d8a939753077792b0554",
                sha224: "3c794f0c67bd561ce841fc6a5999bf0df298a0f0ae3487efda9d0ef4",
                sha256: "6b51d431df5d7f141cbececcf79edf3dd861c3b4069f0b11661a3eefacbba918",
                sha384: "1e237288d39d815abc653befcab0eb70966558a5bbc10a24739c116ed2f615be31e81670f02af48fe3cf5112f0fa03e8",
                sha512: "5aadb45520dcd8726b2822a7a78bb53d794f557199d5d4abdedd2c55a4bd6ca73607605c558de3db80c8e86c3196484566163ed1327e82e8b6757d1932113cb8",
            },
            &TestSeq {
                expr: "HASH(null, cast(value as text), cast((value+1) as blob), null)",
                count: 1000,
                md5: "ddb57ed155427267671e1b525d13e94e",
                sha1: "c22425800e32485e480b8cfc757ec5364d877be2",
                sha224: "03b2734c57c5977bc16d167d0e2851d8c4dc1974dabb85d4ed71778f",
                sha256: "d19ba4af1679ded018a97d9d8b46f15ae5271fa0d323bdf2f0226291b5f8750a",
                sha384: "788ffdde31a515069f4e6b3079f5d29ca4158f0414b9c02cafa9a9b452260c36fa8f6fd0d227b36aa2366c628f3c2fbc",
                sha512: "661875fdb49838bd3e6c34bf51c326d687c32f096dc44f3f4b52dcb44b7c838877aa7405868712d447e34a52f565092bfcdc57853319e91a7ab5e4f087c30589",
            },
        ] {
            let cnv = |v| if v == "None" { None } else { Some(v) };
            #[cfg(feature = "md5")]
            seq(c, &hash(t.expr, "md5_concat"), t.count, cnv(t.md5));
            #[cfg(feature = "sha1")]
            seq(c, &hash(t.expr, "sha1_concat"), t.count, cnv(t.sha1));
            #[cfg(feature = "sha224")]
            seq(c, &hash(t.expr, "sha224_concat"), t.count, cnv(t.sha224));
            #[cfg(feature = "sha256")]
            seq(c, &hash(t.expr, "sha256_concat"), t.count, cnv(t.sha256));
            #[cfg(feature = "sha384")]
            seq(c, &hash(t.expr, "sha384_concat"), t.count, cnv(t.sha384));
            #[cfg(feature = "sha512")]
            seq(c, &hash(t.expr, "sha512_concat"), t.count, cnv(t.sha512));
        }
}

#[test]
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
        #[cfg(feature = "sha224")]
        "sha224",
        #[cfg(feature = "sha224")]
        "sha224_concat",
        #[cfg(feature = "sha256")]
        "sha256",
        #[cfg(feature = "sha256")]
        "sha256_concat",
        #[cfg(feature = "sha384")]
        "sha384",
        #[cfg(feature = "sha384")]
        "sha384_concat",
        #[cfg(feature = "sha512")]
        "sha512",
        #[cfg(feature = "sha512")]
        "sha512_concat",
    ] {
        // NULLs
        is_null(c, &hash("HASH(NULL)", func));
        is_null(c, &hash("HASH(NULL, NULL, NULL)", func));
        // Errors
        is_err(c, &hash("HASH(1)", func));
        is_err(c, &hash("HASH(0.42)", func));
        is_err(c, &hash("HASH()", func));
    }
}
