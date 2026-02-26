use crate::contract;
use wrevive_api::off_chain;

#[test]
fn deploy_sets_owner_and_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([1u8; 20]);
    });
    let _ = contract::deploy();

    assert_eq!(contract::get_owner(), [1u8; 20]);
    assert_eq!(contract::get_value(), 0);
}

#[test]
fn set_value_and_get_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([2u8; 20]);
    });
    let _ = contract::deploy();

    let _ = contract::set_value(42);
    assert_eq!(contract::get_value(), 42);

    let _ = contract::set_value(100);
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
    let _ = contract::deploy();

    // Alice 可以改 owner 为 Bob
    let _ = contract::set_owner(bob, 0);
    assert_eq!(contract::get_owner(), bob);

    // Bob 为 caller 时才能再改 owner；若用 Alice 作为 caller 调用 set_owner 会 revert
    off_chain::with_engine(|e| {
        e.set_caller(bob);
    });
    let new_owner = [3u8; 20];
    let _ = contract::set_owner(new_owner, 0);
    assert_eq!(contract::get_owner(), new_owner);
}

#[test]
fn deposit_event_on_set_value() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([0u8; 20]);
    });
    let _ = contract::deploy();
    let _ = contract::set_value(123);

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
    let _ = contract::deploy();

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
    let _ = contract::deploy();

    // 设置用户信息
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
    let _ = contract::deploy();

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

// ======================== List (RECORDS) 测试 ========================

#[test]
fn records_push_get_len_list() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([1u8; 20]);
    });
    let _ = contract::deploy();

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
        e.set_caller([1u8; 20]);
    });
    let _ = contract::deploy();

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
    let alice = [1u8; 20];
    let bob = [2u8; 20];

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice);
    });
    let _ = contract::deploy();

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
    let alice = [1u8; 20];
    let bob = [2u8; 20];

    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice);
    });
    let _ = contract::deploy();

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
