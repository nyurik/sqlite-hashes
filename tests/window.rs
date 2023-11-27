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

    test_all!(c.window_text_one(*_concat), blob("aaabbbccc"));
    test_all!(
        c.growing_text_seq(*_concat),
        blob[["aaa", "aaabbb", "aaabbbccc"]]
    );

    test_all!(c.window_err(*_concat), ERROR);

    test_all!(c.window_text_zero(*_concat), NO_ROWS);
}

#[test]
#[cfg(feature = "hex")]
fn window_hex() {
    let c = Conn::new();

    test_all!(c.window_text_one(*_concat_hex), hex("aaabbbccc"));

    test_all!(
        c.growing_text_seq(*_concat_hex),
        hex[["aaa", "aaabbb", "aaabbbccc"]]
    );

    test_all!(c.window_text_zero(*_concat_hex), NO_ROWS);
}
