//! 单元测试：对应 ink flipper 的 it_works

use crate::flipper;
use wrevive_api::off_chain;

#[test]
fn it_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([0u8; 20]);
    });
    flipper::deploy();

    assert!(!flipper::get());
    flipper::flip();
    assert!(flipper::get());
    flipper::flip();
    assert!(!flipper::get());
}
