//! 单元测试：对应 ink incrementer 的 default_works 与 it_works

use crate::incrementer;
use wrevive_api::off_chain;

#[test]
fn default_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([0u8; 20]);
    });
    incrementer::deploy();
    assert_eq!(incrementer::get(), 0);
}

#[test]
fn it_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([0u8; 20]);
    });
    incrementer::deploy();
    assert_eq!(incrementer::get(), 0);

    incrementer::inc(42);
    assert_eq!(incrementer::get(), 42);
    incrementer::inc(5);
    assert_eq!(incrementer::get(), 47);
    incrementer::inc(-50);
    assert_eq!(incrementer::get(), -3);
}
