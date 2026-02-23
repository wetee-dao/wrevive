//! 对应 ink contract-storage：set/get 存储

use crate::simple_storage;
use wrevive_api::off_chain;

#[test]
fn set_and_get_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([0u8; 20]);
    });
    simple_storage::deploy();
    assert_eq!(simple_storage::get(0), 0);
    simple_storage::set(0, 42);
    assert_eq!(simple_storage::get(0), 42);
    simple_storage::set(1, 100);
    assert_eq!(simple_storage::get(1), 100);
    assert_eq!(simple_storage::get(0), 42);
}
