#![cfg(feature = "aggregate")]

#[macro_use]
#[path = "_utils.rs"]
mod utils;
use crate::utils::Conn;

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn simple_concat() {
    let c = Conn::new();
    test_all!(c.select("_concat(NULL)"), NULL);
    test_all!(c.select("_concat(NULL, NULL, NULL)"), NULL);
    test_all!(c.select("_concat(1)"), ERROR);
    test_all!(c.select("_concat(0.42)"), ERROR);
    test_all!(c.select("_concat()"), ERROR);
    test_all!(c.select("_concat('')"), blob(""));
    test_all!(c.select("_concat('a')"), blob("a"));
    test_all!(c.select("_concat('123456789')"), blob("123456789"));
    test_all!(c.select("_concat(x'')"), blob(""));
    test_all!(c.select("_concat(x'00')"), blob("\0"));
    test_all!(
        c.select("_concat(x'0123456789abcdef')"),
        bytes_as_blob(b"\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
    test_all!(
        c.select("_concat('', 'a', '123456789', x'', x'00', x'0123456789abcdef')"),
        bytes_as_blob(b"a123456789\x00\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
    test_all!(
        c.select(
            "_concat(NULL, 'a', NULL, '123456789', x'', x'00', NULL, x'0123456789abcdef', NULL)"
        ),
        bytes_as_blob(b"a123456789\x00\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
}

#[test]
#[cfg(feature = "hex")]
fn simple_concat_hex() {
    let c = Conn::new();
    test_all!(c.select("_concat_hex(NULL)"), EMPTY);
    test_all!(c.select("_concat_hex(NULL, NULL, NULL)"), EMPTY);
    test_all!(c.select("_concat_hex(1)"), ERROR);
    test_all!(c.select("_concat_hex(0.42)"), ERROR);
    test_all!(c.select("_concat_hex()"), ERROR);
    test_all!(c.select("_concat_hex('')"), hex(""));
    test_all!(c.select("_concat_hex('a')"), hex("a"));
    test_all!(c.select("_concat_hex('123456789')"), hex("123456789"));
    test_all!(c.select("_concat_hex(x'')"), hex(""));
    test_all!(c.select("_concat_hex(x'00')"), hex("\0"));
    test_all!(
        c.select("_concat_hex(x'0123456789abcdef')"),
        bytes_as_hex(b"\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
    test_all!(
        c.select("_concat_hex('', 'a', '123456789', x'', x'00', x'0123456789abcdef')"),
        bytes_as_hex(b"a123456789\x00\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
    test_all!(
        c.select("_concat_hex(NULL, 'a', NULL, '123456789', x'', x'00', NULL, x'0123456789abcdef', NULL)"),
        bytes_as_hex(b"a123456789\x00\x01\x23\x45\x67\x89\xab\xcd\xef")
    );
}

#[test]
fn hash_concat() {
    let c = Conn::new();
    test_all!(c.legacy_text_aggregate(*_concat), blob("aaabbbccc"));
    test_all!(c.legacy_blob_aggregate(*_concat), blob("aaabbbccc"));
    test_all!(c.legacy_null_text_aggregate(*_concat), NULL);
    test_all!(c.legacy_null_blob_aggregate(*_concat), NULL);
}

#[test]
#[cfg(feature = "hex")]
fn hash_concat_hex() {
    let c = Conn::new();
    test_all!(c.legacy_text_aggregate(*_concat_hex), hex("aaabbbccc"));
}

#[test]
fn concat_sequence() {
    let c = Conn::new();

    test_all!(c.seq_0("_concat(cast(v as text))"), NULL);
    test_all!(c.seq_0("_concat(cast(v as blob))"), NULL);

    test_all!(c.seq_1("_concat(cast(v as text))"), blob("1"));
    test_all!(c.seq_1("_concat(cast(v as blob))"), blob("1"));
    test_all!(
        c.seq_1("_concat(cast(v as text), cast((v+1) as blob))"),
        blob("12")
    );

    let expected = (1..=1000)
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join("");
    test_all!(c.seq_1000("_concat(cast(v as text))"), blob(expected));
    test_all!(c.seq_1000("_concat(cast(v as blob))"), blob(expected));

    let expected = (1..=1000)
        .map(|i| format!("{}{}", i, i + 1))
        .collect::<Vec<String>>()
        .join("");
    test_all!(
        c.seq_1000("_concat(null, cast(v as text), cast((v+1) as blob), null)"),
        blob(expected)
    );
}

#[test]
#[cfg(feature = "hex")]
fn concat_sequence_hex() {
    let c = Conn::new();

    test_all!(c.seq_0("_concat_hex(cast(v as text))"), NULL);
    test_all!(c.seq_0("_concat_hex(cast(v as blob))"), NULL);

    test_all!(c.seq_1("_concat_hex(cast(NULL as text))"), EMPTY);
    test_all!(c.seq_1("_concat_hex(cast(NULL as blob))"), EMPTY);

    test_all!(c.seq_1("_concat_hex(cast(v as text))"), hex("1"));
    test_all!(c.seq_1("_concat_hex(cast(v as blob))"), hex("1"));
    test_all!(
        c.seq_1("_concat_hex(cast(v as text), cast((v+1) as blob))"),
        hex("12")
    );

    let expected = (1..=1000)
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join("");
    test_all!(c.seq_1000("_concat_hex(cast(v as text))"), hex(expected));
    test_all!(c.seq_1000("_concat_hex(cast(v as blob))"), hex(expected));

    let expected = (1..=1000)
        .map(|i| format!("{}{}", i, i + 1))
        .collect::<Vec<String>>()
        .join("");
    test_all!(
        c.seq_1000("_concat_hex(null, cast(v as text), cast((v+1) as blob), null)"),
        hex(expected)
    );
}
