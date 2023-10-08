#![cfg(feature = "window")]

#[macro_use]
#[path = "_utils.rs"]
mod utils;
use crate::utils::Conn;

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn window() {
    let c = Conn::new();

    test_all!(c.window_one(*_concat), blob("aaabbbccc"));

    test_all!(
        c.growing_seq(*_concat),
        blob[vec!["aaa", "aaabbb", "aaabbbccc"]]
    );

    test_all!(c.window_err(*_concat), ERROR);
}

#[test]
#[cfg(feature = "hex")]
fn window_hex() {
    let c = Conn::new();

    test_all!(c.window_one(*_concat_hex), hex("aaabbbccc"));

    test_all!(
        c.growing_seq(*_concat_hex),
        hex[vec!["aaa", "aaabbb", "aaabbbccc"]]
    );
}
