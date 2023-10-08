#![allow(dead_code)]

#[macro_use]
#[path = "_utils.rs"]
mod utils;
use crate::utils::Conn;

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn simple() {
    let c = Conn::new();
    test_all!(c.select("(NULL)"), NULL);
    test_all!(c.select("(NULL, NULL, NULL)"), NULL);
    test_all!(c.select("(1)"), ERROR);
    test_all!(c.select("(0.42)"), ERROR);
    test_all!(c.select("()"), ERROR);
    test_all!(c.select("('')"), blob(""));
    test_all!(c.select("('a')"), blob("a"));
    test_all!(c.select("('123456789')"), blob("123456789"));
    test_all!(c.select("(x'')"), blob(""));
    test_all!(c.select("(x'00')"), blob("\0"));
    test_all!(
        c.select("(x'0123456789abcdef')"),
        bytes_as_blob(b"\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
    test_all!(
        c.select("('', 'a', '123456789', x'', x'00', x'0123456789abcdef')"),
        bytes_as_blob(b"a123456789\x00\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
    test_all!(
        c.select("(NULL, 'a', NULL, '123456789', x'', x'00', NULL, x'0123456789abcdef', NULL)"),
        bytes_as_blob(b"a123456789\x00\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
}

#[test]
#[cfg(feature = "hex")]
fn simple_hex() {
    let c = Conn::new();
    test_all!(c.select("_hex(NULL)"), NULL);
    test_all!(c.select("_hex(NULL, NULL, NULL)"), NULL);
    test_all!(c.select("_hex(1)"), ERROR);
    test_all!(c.select("_hex(0.42)"), ERROR);
    test_all!(c.select("_hex()"), ERROR);
    test_all!(c.select("_hex('')"), hex(""));
    test_all!(c.select("_hex('a')"), hex("a"));
    test_all!(c.select("_hex('123456789')"), hex("123456789"));
    test_all!(c.select("_hex(x'')"), hex(""));
    test_all!(c.select("_hex(x'00')"), hex("\0"));
    test_all!(
        c.select("_hex(x'0123456789abcdef')"),
        bytes_as_hex(b"\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
    test_all!(
        c.select("_hex('', 'a', '123456789', x'', x'00', x'0123456789abcdef')"),
        bytes_as_hex(b"a123456789\x00\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
    test_all!(
        c.select("_hex(NULL, 'a', NULL, '123456789', x'', x'00', NULL, x'0123456789abcdef', NULL)"),
        bytes_as_hex(b"a123456789\x00\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
}
