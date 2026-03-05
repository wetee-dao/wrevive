//! Unit tests for the codec example contract. Use off_chain Engine for storage/caller/events.
//! 示例合约单元测试；使用 off_chain Engine 模拟存储、调用者与事件。

use crate::contract;
use wrevive_api::{off_chain, Address};

/// Deploy sets caller as owner and value to 0. 部署后 caller 为 owner，value 为 0。
#[test]
fn deploy_sets_owner_and_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([1u8; 20]).into());
    });
    let _ = contract::deploy(0);

    assert_eq!(contract::get_owner(), Address::from([1u8; 20]));
    assert_eq!(contract::get_value(), 0);
}

/// Deploy with initial_value sets VALUE to that. 带参部署时 VALUE 为传入的 initial_value。
#[test]
fn deploy_with_initial_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([5u8; 20]).into());
    });
    let _ = contract::deploy(100);
    assert_eq!(contract::get_value(), 100);
    let _ = contract::deploy(200);
    assert_eq!(contract::get_value(), 200);
}

#[test]
fn set_value_and_get_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([2u8; 20]).into());
    });
    let _ = contract::deploy(0);

    let _ = contract::set_value(42);
    assert_eq!(contract::get_value(), 42);

    let _ = contract::set_value(100);
    assert_eq!(contract::get_value(), 100);
}

#[test]
fn set_owner_only_by_current_owner() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);

    // Alice 可以改 owner 为 Bob
    let _ = contract::set_owner(bob);
    assert_eq!(contract::get_owner(), bob);

    // Bob 为 caller 时才能再改 owner；若用 Alice 作为 caller 调用 set_owner 会 revert
    off_chain::with_engine(|e| {
        e.set_caller(bob.into());
    });
    let new_owner = Address::from([3u8; 20]);
    let _ = contract::set_owner(new_owner);
    assert_eq!(contract::get_owner(), new_owner);
}

#[test]
fn deposit_event_on_set_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([0u8; 20]).into());
    });
    let _ = contract::deploy(0);
    let _ = contract::set_value(123);

    off_chain::with_engine(|e| {
        let events = e.take_events();
        // deploy + set_value 都会触发事件，这里检查最后一条事件的 payload 是否为 123
        let last = events.last().expect("at least one event");
        assert_eq!(last.1, 123u32.to_le_bytes());
    });
}

#[test]
fn mapping_balance_works() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);

    // 设置 Alice 的余额
    let _ = contract::set_balance(alice, 1000);
    assert_eq!(contract::get_balance(alice), 1000);

    // 设置 Bob 的余额
    let _ = contract::set_balance(bob, 500);
    assert_eq!(contract::get_balance(bob), 500);

    // 初始余额为 0
    let charlie = Address::from([3u8; 20]);
    assert_eq!(contract::get_balance(charlie), 0);
}

#[test]
fn mapping_user_info_works() {
    let alice = Address::from([1u8; 20]);

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);

    // 设置用户信息
    let _ = contract::set_user_info(alice, 0, 25);
    assert_eq!(contract::get_user_info(alice, 0), 25);

    // 设置用户信息：info_type 1 = 积分
    let _ = contract::set_user_info(alice, 1, 100);
    assert_eq!(contract::get_user_info(alice, 1), 100);

    // 设置用户信息：info_type 2 = 等级
    let _ = contract::set_user_info(alice, 2, 5);
    assert_eq!(contract::get_user_info(alice, 2), 5);

    // 不同 info_type 的值互不影响
    assert_eq!(contract::get_user_info(alice, 0), 25);
    assert_eq!(contract::get_user_info(alice, 1), 100);
}

#[test]
fn mapping_transfer_balance_works() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);

    // 设置初始余额
    let _ = contract::set_balance(alice, 1000);
    let _ = contract::set_balance(bob, 500);

    // 转账 200 从 Alice 到 Bob
    let _ = contract::transfer_balance(alice, bob, 200);
    assert_eq!(contract::get_balance(alice), 800);
    assert_eq!(contract::get_balance(bob), 700);

    // 再次转账 100
    let _ = contract::transfer_balance(alice, bob, 100);
    assert_eq!(contract::get_balance(alice), 700);
    assert_eq!(contract::get_balance(bob), 800);
}

/// Insufficient balance must revert (off_chain: panic). 余额不足必须 revert（链下会 panic）。
#[test]
fn transfer_balance_insufficient_balance_reverts() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);
    let _ = contract::set_balance(alice, 100);
    let res = contract::transfer_balance(alice, bob, 200);
    assert_eq!(res, Err(contract::Error::InsufficientBalance));
}

/// Only `from` may call transfer_balance; else revert (off_chain: panic). 仅 from 可调用转账，否则 revert。
#[test]
fn transfer_balance_only_from_may_call() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(bob.into());
    });
    let _ = contract::deploy(0);
    let _ = contract::set_balance(alice, 1000);
    let res = contract::transfer_balance(alice, bob, 100);
    assert_eq!(res, Err(contract::Error::Unauthorized));
}

// ======================== 异常 / 边界：owner 与 value ========================

/// Non-owner calling set_owner must revert. 非 owner 调用 set_owner 必须 revert。
#[test]
fn set_owner_reverts_when_not_owner() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);
    let charlie = Address::from([3u8; 20]);
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);
    off_chain::with_engine(|e| e.set_caller(bob.into()));
    let res = contract::set_owner(charlie);
    assert_eq!(res, Err(contract::Error::Unauthorized));
}

/// get_value returns 0 when never set (after deploy). 未设置时 get_value 返回 0。
#[test]
fn get_value_default_before_any_set() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([5u8; 20]).into());
    });
    let _ = contract::deploy(0);
    assert_eq!(contract::get_value(), 0);
}

/// get_owner returns zero address when never set (e.g. storage cleared). 未设置时 get_owner 返回零地址。
#[test]
fn get_owner_after_deploy_is_caller() {
    let addr = Address::from([7u8; 20]);
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(addr.into());
    });
    let _ = contract::deploy(0);
    assert_eq!(contract::get_owner(), addr);
}

// ======================== 异常 / 边界：List (records) ========================

#[test]
fn records_list_size_zero_returns_empty() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([1u8; 20]).into());
    });
    let _ = contract::deploy(0);
    contract::records_push(1);
    let empty = contract::records_list(0, 0);
    assert!(empty.is_empty());
}

#[test]
fn records_list_start_beyond_len_returns_empty() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([1u8; 20]).into());
    });
    let _ = contract::deploy(0);
    contract::records_push(10);
    contract::records_push(20);
    assert_eq!(contract::records_len(), 2);
    let empty = contract::records_list(10, 5);
    assert!(empty.is_empty());
}

#[test]
fn records_get_nonexistent_id_returns_zero() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([1u8; 20]).into());
    });
    let _ = contract::deploy(0);
    assert_eq!(contract::records_get(0), 0);
    assert_eq!(contract::records_get(999), 0);
}

// ======================== 异常 / 边界：List2D (user_items) ========================

#[test]
fn user_items_list_empty_user_returns_empty() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);
    let empty = contract::user_items_list(bob, 0, 10);
    assert!(empty.is_empty());
    assert_eq!(contract::user_items_len(bob), 0);
}

#[test]
fn user_items_list_size_zero_returns_empty() {
    let alice = Address::from([1u8; 20]);
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);
    contract::user_items_push(alice, 1);
    let empty = contract::user_items_list(alice, 0, 0);
    assert!(empty.is_empty());
}

#[test]
fn user_items_get_nonexistent_k2_returns_zero() {
    let alice = Address::from([1u8; 20]);
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);
    contract::user_items_push(alice, 42);
    assert_eq!(contract::user_items_get(alice, 99), 0);
}

// ======================== 异常 / 边界：transfer ========================

/// Transfer amount 0 is allowed (no-op). 转账 0 允许（无操作）。
#[test]
fn transfer_balance_amount_zero_allowed() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);
    let _ = contract::set_balance(alice, 1000);
    let _ = contract::set_balance(bob, 500);
    let _ = contract::transfer_balance(alice, bob, 0);
    assert_eq!(contract::get_balance(alice), 1000);
    assert_eq!(contract::get_balance(bob), 500);
}

/// Self-transfer (from == to) is allowed when caller is from. 自己转给自己允许（caller 为 from）。
#[test]
fn transfer_balance_self_transfer_allowed() {
    let alice = Address::from([1u8; 20]);
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);
    let _ = contract::set_balance(alice, 1000);
    let _ = contract::transfer_balance(alice, alice, 100);
    assert_eq!(contract::get_balance(alice), 1000);
}

// ======================== 异常 / 边界：user_info ========================

#[test]
fn get_user_info_never_set_returns_zero() {
    let alice = Address::from([1u8; 20]);
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);
    assert_eq!(contract::get_user_info(alice, 0), 0);
    assert_eq!(contract::get_user_info(alice, 255), 0);
}

// ======================== List (RECORDS) tests / List 测试 ========================

#[test]
fn records_push_get_len_list() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([1u8; 20]).into());
    });
    let _ = contract::deploy(0);

    assert_eq!(contract::records_len(), 0);

    let id0 = contract::records_push(100);
    assert_eq!(id0, Some(0));
    assert_eq!(contract::records_get(0), 100);
    assert_eq!(contract::records_len(), 1);

    let id1 = contract::records_push(200);
    assert_eq!(id1, Some(1));
    assert_eq!(contract::records_get(1), 200);
    assert_eq!(contract::records_len(), 2);

    contract::records_push(300);
    assert_eq!(contract::records_get(2), 300);
    assert_eq!(contract::records_len(), 3);

    // 不存在的 id 返回 0
    assert_eq!(contract::records_get(99), 0);
}

#[test]
fn records_list_pagination() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([1u8; 20]).into());
    });
    let _ = contract::deploy(0);

    for v in [10u64, 20, 30, 40, 50] {
        contract::records_push(v);
    }
    assert_eq!(contract::records_len(), 5);

    let all = contract::records_list(0, 10);
    assert_eq!(all, [(0, 10), (1, 20), (2, 30), (3, 40), (4, 50)]);

    let page1 = contract::records_list(0, 2);
    assert_eq!(page1, [(0, 10), (1, 20)]);

    let page2 = contract::records_list(2, 2);
    assert_eq!(page2, [(2, 30), (3, 40)]);

    let page3 = contract::records_list(4, 5);
    assert_eq!(page3, [(4, 50)]);

    let empty = contract::records_list(10, 5);
    assert!(empty.is_empty());
}

// ======================== List2D (USER_ITEMS) 测试 ========================

#[test]
fn user_items_push_get_len_list() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);

    assert_eq!(contract::user_items_len(alice), 0);
    assert_eq!(contract::user_items_len(bob), 0);

    let k0 = contract::user_items_push(alice, 11);
    assert_eq!(k0, Some(0));
    assert_eq!(contract::user_items_get(alice, 0), 11);
    assert_eq!(contract::user_items_len(alice), 1);

    contract::user_items_push(alice, 22);
    assert_eq!(contract::user_items_get(alice, 1), 22);
    assert_eq!(contract::user_items_len(alice), 2);

    // Bob 的列表独立
    contract::user_items_push(bob, 99);
    assert_eq!(contract::user_items_get(bob, 0), 99);
    assert_eq!(contract::user_items_len(bob), 1);
    assert_eq!(contract::user_items_len(alice), 2);

    // 不存在的 k2 返回 0
    assert_eq!(contract::user_items_get(alice, 99), 0);
}

#[test]
fn user_items_list_pagination() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);

    for v in [1u32, 2, 3, 4, 5] {
        contract::user_items_push(alice, v);
    }
    contract::user_items_push(bob, 100);
    contract::user_items_push(bob, 200);

    let alice_all = contract::user_items_list(alice, 0, 10);
    assert_eq!(alice_all, [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]);

    let alice_page = contract::user_items_list(alice, 1, 2);
    assert_eq!(alice_page, [(1, 2), (2, 3)]);

    let bob_all = contract::user_items_list(bob, 0, 10);
    assert_eq!(bob_all, [(0, 100), (1, 200)]);

    let empty = contract::user_items_list(alice, 10, 5);
    assert!(empty.is_empty());
}

// ======================== Solidity-style messages (set_value_sol / get_value_sol) ========================

/// set_value_sol and get_value_sol behave like set_value/get_value (same storage).
#[test]
fn set_value_sol_get_value_sol_work() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([1u8; 20]).into());
    });
    let _ = contract::deploy(0);

    let _ = contract::set_value_sol(77);
    assert_eq!(contract::get_value_sol(), 77);

    let _ = contract::set_value_sol(999);
    assert_eq!(contract::get_value_sol(), 999);

    // Shared storage with set_value/get_value
    let _ = contract::set_value(111);
    assert_eq!(contract::get_value_sol(), 111);
    assert_eq!(contract::get_value(), 111);
}

// ======================== Deploy event ========================

/// deploy deposits an event with initial_value as payload.
#[test]
fn deploy_deposits_event_with_initial_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([0u8; 20]).into());
    });
    let _ = contract::deploy(123);

    off_chain::with_engine(|e| {
        let events = e.take_events();
        assert!(!events.is_empty(), "deploy should emit at least one event");
        let deploy_event = &events[0];
        assert_eq!(deploy_event.1, 123u32.to_le_bytes());
    });
}

// ======================== Records edge: empty list ========================

/// records_len is 0 before any records_push.
#[test]
fn records_len_zero_before_any_push() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(Address::from([1u8; 20]).into());
    });
    let _ = contract::deploy(0);
    assert_eq!(contract::records_len(), 0);
}

// ======================== User items edge: multiple users isolated ========================

/// user_items for different users are isolated; len per user is correct.
#[test]
fn user_items_multiple_users_isolated() {
    let alice = Address::from([1u8; 20]);
    let bob = Address::from([2u8; 20]);
    let charlie = Address::from([3u8; 20]);

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice.into());
    });
    let _ = contract::deploy(0);

    contract::user_items_push(alice, 1);
    contract::user_items_push(alice, 2);
    contract::user_items_push(bob, 10);
    contract::user_items_push(charlie, 20);
    contract::user_items_push(charlie, 21);

    assert_eq!(contract::user_items_len(alice), 2);
    assert_eq!(contract::user_items_len(bob), 1);
    assert_eq!(contract::user_items_len(charlie), 2);

    assert_eq!(contract::user_items_get(alice, 0), 1);
    assert_eq!(contract::user_items_get(alice, 1), 2);
    assert_eq!(contract::user_items_get(bob, 0), 10);
    assert_eq!(contract::user_items_get(charlie, 0), 20);
    assert_eq!(contract::user_items_get(charlie, 1), 21);
}
