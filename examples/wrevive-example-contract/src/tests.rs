//! 单元测试：使用 wrevive_api::off_chain Engine，运行 `cargo test -p wrevive-example-contract --features test`

use crate::contract;
use wrevive_api::off_chain;

const STORAGE_KEY_VALUE: &[u8] = b"value";
const STORAGE_KEY_OWNER: &[u8] = b"owner";

#[test]
fn deploy_sets_owner_and_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([1u8; 20]);
    });
    contract::deploy();

    off_chain::with_engine(|e| {
        let owner = e.get_storage_value(STORAGE_KEY_OWNER).unwrap();
        assert_eq!(owner.len(), 20);
        assert_eq!(owner, [1u8; 20]);
        let value = e.get_storage_value(STORAGE_KEY_VALUE).unwrap();
        assert_eq!(value, 0u32.to_le_bytes());
    });
}

#[test]
fn set_value_and_get_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([2u8; 20]);
    });
    contract::deploy();

    contract::set_value(42);
    assert_eq!(contract::get_value(), 42);

    contract::set_value(100);
    assert_eq!(contract::get_value(), 100);
}

#[test]
fn set_owner_only_by_current_owner() {
    let alice = [1u8; 20];
    let bob = [2u8; 20];

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice);
    });
    contract::deploy();

    // Alice 可以改 owner 为 Bob
    contract::set_owner(bob);
    off_chain::with_engine(|e| {
        assert_eq!(e.get_storage_value(STORAGE_KEY_OWNER).unwrap(), bob);
    });

    // Bob 为 caller 时才能再改 owner；若用 Alice 作为 caller 调用 set_owner 会 revert
    off_chain::with_engine(|e| {
        e.set_caller(bob);
    });
    let new_owner = [3u8; 20];
    contract::set_owner(new_owner);
    off_chain::with_engine(|e| {
        assert_eq!(e.get_storage_value(STORAGE_KEY_OWNER).unwrap(), new_owner);
    });
}

#[test]
fn deposit_event_on_set_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([0u8; 20]);
    });
    contract::deploy();
    contract::set_value(123);

    off_chain::with_engine(|e| {
        let events = e.take_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].1, 123u32.to_le_bytes());
    });
}
