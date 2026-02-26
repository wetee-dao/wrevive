//! API unit tests: off_chain Engine for storage, caller, Mapping, List, List2D.
//! API 单元测试：使用 off_chain Engine 测试存储、caller、Mapping、List、List2D。

use super::{env, off_chain, List, List2D, Mapping, StorageFlags};

#[test]
fn off_chain_engine_storage_and_caller() {
    off_chain::with_engine(|e| {
        e.set_caller([1u8; 20]);
        e.set_call_data(&[]);
    });
    let caller = env().caller();
    assert_eq!(caller, [1u8; 20]);

    env().set_storage_bytes(StorageFlags::empty(), b"key", b"value");
    off_chain::with_engine(|e| {
        let v = e.get_storage_value(b"key").unwrap();
        assert_eq!(v, b"value");
    });
}

#[test]
fn mapping_set_and_get() {
    off_chain::with_engine(|e| e.reset());
    let m: Mapping<[u8; 20], u64> = Mapping::new(b"balance");
    let alice = [1u8; 20];
    let bob = [2u8; 20];
    let e = env();
    m.set(e, &alice, &1000u64).unwrap();
    m.set(e, &bob, &2000u64).unwrap();

    let v: u64 = m.get(e, &alice).unwrap();
    assert_eq!(v, 1000);
    let v2: u64 = m.get(e, &bob).unwrap();
    assert_eq!(v2, 2000);

    let m_val: Mapping<(u32, [u8; 3]), Vec<u8>> = Mapping::new(b"val");
    m_val.set(e, &(42u32, *b"sub"), &b"val42".to_vec()).unwrap();
    let out: Vec<u8> = m_val.get(e, &(42u32, *b"sub")).unwrap();
    assert_eq!(&out[..], b"val42");

    let m2: Mapping<u32, u64> = Mapping::new(b"cnt");
    m2.set(e, &1u32, &100u64).unwrap();
    m2.set(e, &2u32, &200u64).unwrap();
    assert_eq!(m2.get(e, &1u32).unwrap(), 100);
    assert_eq!(m2.get(e, &2u32).unwrap(), 200);
}

// ======================== List 单元测试 ========================

#[test]
fn list_u32_len_empty() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u32, u64> = List::new(b"list_t1_next", b"list_t1_items");
    let e = env();
    assert_eq!(list.len(e), 0u32);
}

#[test]
fn list_u32_insert_and_get() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u32, u64> = List::new(b"list_t2_next", b"list_t2_items");
    let e = env();
    let k0 = list.insert(e, &100u64).unwrap();
    assert_eq!(k0, 0);
    assert_eq!(list.len(e), 1);
    assert_eq!(list.get(e, &0).unwrap(), 100);

    let k1 = list.insert(e, &200u64).unwrap();
    assert_eq!(k1, 1);
    assert_eq!(list.len(e), 2);
    assert_eq!(list.get(e, &1).unwrap(), 200);

    let k2 = list.insert(e, &300u64).unwrap();
    assert_eq!(k2, 2);
    assert_eq!(list.get(e, &2).unwrap(), 300);
}

#[test]
fn list_u32_contains() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u32, String> = List::new(b"list_t3_next", b"list_t3_items");
    let e = env();
    assert!(!list.contains(e, &0));
    list.insert(e, &"a".to_string()).unwrap();
    assert!(list.contains(e, &0));
    assert!(!list.contains(e, &1));
    list.insert(e, &"b".to_string()).unwrap();
    assert!(list.contains(e, &1));
}

#[test]
fn list_u32_update() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u32, u64> = List::new(b"list_t4_next", b"list_t4_items");
    let e = env();
    list.insert(e, &10u64).unwrap();
    list.insert(e, &20u64).unwrap();
    assert_eq!(list.get(e, &0).unwrap(), 10);
    list.update(e, &0, &99u64).unwrap();
    assert_eq!(list.get(e, &0).unwrap(), 99);
    assert_eq!(list.get(e, &1).unwrap(), 20);
}

#[test]
fn list_u32_list_asc() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u32, u64> = List::new(b"list_t5_next", b"list_t5_items");
    let e = env();
    for v in [1u64, 2, 3, 4, 5] {
        list.insert(e, &v).unwrap();
    }
    let page = list.list(e, 0, 3);
    assert_eq!(page.len(), 3);
    assert_eq!(page[0], (0, 1));
    assert_eq!(page[1], (1, 2));
    assert_eq!(page[2], (2, 3));

    let page2 = list.list(e, 2, 10);
    assert_eq!(page2.len(), 3);
    assert_eq!(page2[0], (2, 3));
    assert_eq!(page2[1], (3, 4));
    assert_eq!(page2[2], (4, 5));

    let page_empty = list.list(e, 0, 0);
    assert!(page_empty.is_empty());
    let page_beyond = list.list(e, 10, 5);
    assert!(page_beyond.is_empty());
}

#[test]
fn list_u32_desc_list() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u32, u64> = List::new(b"list_t6_next", b"list_t6_items");
    let e = env();
    for v in [10u64, 20, 30, 40] {
        list.insert(e, &v).unwrap();
    }
    let desc = list.desc_list(e, None, 3);
    assert_eq!(desc.len(), 3);
    assert_eq!(desc[0], (3, 40));
    assert_eq!(desc[1], (2, 30));
    assert_eq!(desc[2], (1, 20));

    let desc_from = list.desc_list(e, Some(2), 2);
    assert_eq!(desc_from.len(), 2);
    assert_eq!(desc_from[0], (2, 30));
    assert_eq!(desc_from[1], (1, 20));

    let desc_empty = list.desc_list(e, None, 0);
    assert!(desc_empty.is_empty());
}

#[test]
fn list_u64_id_type() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u64, u32> = List::new(b"list_t7_next", b"list_t7_items");
    let e = env();
    assert_eq!(list.len(e), 0u64);
    let k = list.insert(e, &42u32).unwrap();
    assert_eq!(k, 0u64);
    assert_eq!(list.len(e), 1u64);
    assert_eq!(list.get(e, &0u64).unwrap(), 42);
}

#[test]
fn list_u8_id_type() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u8, i32> = List::new(b"list_t8_next", b"list_t8_items");
    let e = env();
    list.insert(e, &1).unwrap();
    list.insert(e, &2).unwrap();
    assert_eq!(list.len(e), 2u8);
    assert_eq!(list.list(e, 0u8, 2).len(), 2);
}

// ======================== List2D 单元测试 ========================

#[test]
fn list_2d_len_empty() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_t1_k1", b"dl_t1_len", b"dl_t1_k2", b"dl_t1_store");
    let e = env();
    let alice = [1u8; 20];
    assert_eq!(dl.len(e, &alice), 0u32);
    assert_eq!(dl.next_id(e, &alice), 0u32);
}

#[test]
fn list_2d_insert_and_get() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_t2_k1", b"dl_t2_len", b"dl_t2_k2", b"dl_t2_store");
    let e = env();
    let alice = [1u8; 20];
    let bob = [2u8; 20];

    let k2_0 = dl.insert(e, &alice, &100u64).unwrap();
    assert_eq!(k2_0, 0);
    assert_eq!(dl.get(e, &alice, 0).unwrap(), 100);
    assert_eq!(dl.len(e, &alice), 1);

    let k2_1 = dl.insert(e, &alice, &200u64).unwrap();
    assert_eq!(k2_1, 1);
    assert_eq!(dl.get(e, &alice, 1).unwrap(), 200);
    assert_eq!(dl.len(e, &alice), 2);

    let k2_bob = dl.insert(e, &bob, &999u64).unwrap();
    assert_eq!(k2_bob, 0);
    assert_eq!(dl.get(e, &bob, 0).unwrap(), 999);
    assert_eq!(dl.len(e, &bob), 1);
    assert_eq!(dl.len(e, &alice), 2);
}

#[test]
fn list_2d_update() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_t3_k1", b"dl_t3_len", b"dl_t3_k2", b"dl_t3_store");
    let e = env();
    let alice = [1u8; 20];
    dl.insert(e, &alice, &10u64).unwrap();
    dl.insert(e, &alice, &20u64).unwrap();
    assert_eq!(dl.get(e, &alice, 0).unwrap(), 10);
    dl.update(e, &alice, 0, &100u64).unwrap();
    assert_eq!(dl.get(e, &alice, 0).unwrap(), 100);
    assert_eq!(dl.get(e, &alice, 1).unwrap(), 20);
}

#[test]
fn list_2d_list_asc() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_t4_k1", b"dl_t4_len", b"dl_t4_k2", b"dl_t4_store");
    let e = env();
    let alice = [1u8; 20];
    for v in [1u64, 2, 3, 4, 5] {
        dl.insert(e, &alice, &v).unwrap();
    }
    let page = dl.list(e, &alice, 0, 3);
    assert_eq!(page.len(), 3);
    assert_eq!(page[0], (0, 1));
    assert_eq!(page[1], (1, 2));
    assert_eq!(page[2], (2, 3));

    let page2 = dl.list(e, &alice, 2, 10);
    assert_eq!(page2.len(), 3);
    assert_eq!(page2[0], (2, 3));
    assert_eq!(page2[1], (3, 4));
    assert_eq!(page2[2], (4, 5));

    let bob = [2u8; 20];
    let empty = dl.list(e, &bob, 0, 5);
    assert!(empty.is_empty());
}

#[test]
fn list_2d_desc_list() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_t5_k1", b"dl_t5_len", b"dl_t5_k2", b"dl_t5_store");
    let e = env();
    let alice = [1u8; 20];
    for v in [10u64, 20, 30, 40] {
        dl.insert(e, &alice, &v).unwrap();
    }
    let desc = dl.desc_list(e, &alice, None, 3);
    assert_eq!(desc.len(), 3);
    assert_eq!(desc[0], (3, 40));
    assert_eq!(desc[1], (2, 30));
    assert_eq!(desc[2], (1, 20));

    let desc_from = dl.desc_list(e, &alice, Some(2), 2);
    assert_eq!(desc_from.len(), 2);
    assert_eq!(desc_from[0], (2, 30));
    assert_eq!(desc_from[1], (1, 20));
}

#[test]
fn list_2d_list_all() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_t6_k1", b"dl_t6_len", b"dl_t6_k2", b"dl_t6_store");
    let e = env();
    let alice = [1u8; 20];
    assert!(dl.list_all(e, &alice).is_empty());
    for v in [7u64, 8, 9] {
        dl.insert(e, &alice, &v).unwrap();
    }
    let all = dl.list_all(e, &alice);
    assert_eq!(all.len(), 3);
    assert_eq!(all[0], (0, 7));
    assert_eq!(all[1], (1, 8));
    assert_eq!(all[2], (2, 9));
}

#[test]
fn list_2d_multiple_k1() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, String> =
        List2D::new(b"dl_t7_k1", b"dl_t7_len", b"dl_t7_k2", b"dl_t7_store");
    let e = env();
    let a = [1u8; 20];
    let b = [2u8; 20];
    let c = [3u8; 20];

    dl.insert(e, &a, &"a1".to_string()).unwrap();
    dl.insert(e, &a, &"a2".to_string()).unwrap();
    dl.insert(e, &b, &"b1".to_string()).unwrap();
    dl.insert(e, &c, &"c1".to_string()).unwrap();
    dl.insert(e, &c, &"c2".to_string()).unwrap();
    dl.insert(e, &c, &"c3".to_string()).unwrap();

    assert_eq!(dl.len(e, &a), 2u32);
    assert_eq!(dl.len(e, &b), 1u32);
    assert_eq!(dl.len(e, &c), 3u32);

    assert_eq!(dl.get(e, &a, 0).as_deref(), Some("a1"));
    assert_eq!(dl.get(e, &a, 1).as_deref(), Some("a2"));
    assert_eq!(dl.get(e, &b, 0).as_deref(), Some("b1"));
    assert_eq!(dl.get(e, &c, 0).as_deref(), Some("c1"));
    assert_eq!(dl.get(e, &c, 2).as_deref(), Some("c3"));

    let all_a = dl.list_all(e, &a);
    assert_eq!(all_a.len(), 2);
    let all_c = dl.list_all(e, &c);
    assert_eq!(all_c.len(), 3);
}

#[test]
fn list_2d_u16_inner_id() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<u32, u16, u8> =
        List2D::new(b"dl_t8_k1", b"dl_t8_len", b"dl_t8_k2", b"dl_t8_store");
    let e = env();
    let k2 = dl.insert(e, &1u32, &10u8).unwrap();
    assert_eq!(k2, 0u16);
    assert_eq!(dl.get(e, &1u32, 0u16).unwrap(), 10);
    assert_eq!(dl.len(e, &1u32), 1u16);
}