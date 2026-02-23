use crate::contract;
use wrevive_api::{get_storage, off_chain};

const STORAGE_KEY_VALUE: &[u8] = b"value";
const STORAGE_KEY_OWNER: &[u8] = b"owner";

#[test]
fn deploy_sets_owner_and_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([1u8; 20]);
    });
    contract::deploy();

    let owner: [u8; 20] = get_storage(wrevive_api::StorageFlags::empty(), STORAGE_KEY_OWNER).unwrap();
    assert_eq!(owner, [1u8; 20]);
    let value: u32 = get_storage(wrevive_api::StorageFlags::empty(), STORAGE_KEY_VALUE).unwrap();
    assert_eq!(value, 0);
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
    contract::set_owner(bob, 0);
    let stored_owner: [u8; 20] = get_storage(wrevive_api::StorageFlags::empty(), STORAGE_KEY_OWNER).unwrap();
    assert_eq!(stored_owner, bob);

    // Bob 为 caller 时才能再改 owner；若用 Alice 作为 caller 调用 set_owner 会 revert
    off_chain::with_engine(|e| {
        e.set_caller(bob);
    });
    let new_owner = [3u8; 20];
    contract::set_owner(new_owner, 0);
    let stored_owner: [u8; 20] = get_storage(wrevive_api::StorageFlags::empty(), STORAGE_KEY_OWNER).unwrap();
    assert_eq!(stored_owner, new_owner);
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

#[test]
fn mapping_balance_works() {
    let alice = [1u8; 20];
    let bob = [2u8; 20];

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice);
    });
    contract::deploy();

    // 设置 Alice 的余额
    contract::set_balance(alice, 1000);
    assert_eq!(contract::get_balance(alice), 1000);

    // 设置 Bob 的余额
    contract::set_balance(bob, 500);
    assert_eq!(contract::get_balance(bob), 500);

    // 初始余额为 0
    let charlie = [3u8; 20];
    assert_eq!(contract::get_balance(charlie), 0);
}

#[test]
fn mapping_user_info_works() {
    let alice = [1u8; 20];

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice);
    });
    contract::deploy();

    // 设置用户信息：info_type 0 = 年龄
    contract::set_user_info(alice, 0, 25);
    assert_eq!(contract::get_user_info(alice, 0), 25);

    // 设置用户信息：info_type 1 = 积分
    contract::set_user_info(alice, 1, 100);
    assert_eq!(contract::get_user_info(alice, 1), 100);

    // 设置用户信息：info_type 2 = 等级
    contract::set_user_info(alice, 2, 5);
    assert_eq!(contract::get_user_info(alice, 2), 5);

    // 不同 info_type 的值互不影响
    assert_eq!(contract::get_user_info(alice, 0), 25);
    assert_eq!(contract::get_user_info(alice, 1), 100);
}

#[test]
fn mapping_transfer_balance_works() {
    let alice = [1u8; 20];
    let bob = [2u8; 20];

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice);
    });
    contract::deploy();

    // 设置初始余额
    contract::set_balance(alice, 1000);
    contract::set_balance(bob, 500);

    // 转账 200 从 Alice 到 Bob
    contract::transfer_balance(alice, bob, 200);
    assert_eq!(contract::get_balance(alice), 800);
    assert_eq!(contract::get_balance(bob), 700);

    // 再次转账 100
    contract::transfer_balance(alice, bob, 100);
    assert_eq!(contract::get_balance(alice), 700);
    assert_eq!(contract::get_balance(bob), 800);
}
