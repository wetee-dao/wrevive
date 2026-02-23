//! 对应 ink trait-incrementer 的 default_works 与 it_works

use crate::trait_incrementer;
use wrevive_api::off_chain;

#[test]
fn default_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([0u8; 20]);
    });
    trait_incrementer::deploy();
    assert_eq!(trait_incrementer::get(), 0);
}

#[test]
fn it_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([0u8; 20]);
    });
    trait_incrementer::deploy();
    assert_eq!(trait_incrementer::get(), 0);

    trait_incrementer::inc();
    assert_eq!(trait_incrementer::get(), 1);

    trait_incrementer::inc_by(10);
    assert_eq!(trait_incrementer::get(), 11);

    trait_incrementer::reset();
    assert_eq!(trait_incrementer::get(), 0);
}
