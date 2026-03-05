//! API unit tests: off_chain Engine for storage, caller, Mapping, List, List2D.
//! API 单元测试：使用 off_chain Engine 测试存储、caller、Mapping、List、List2D。

use super::{env, off_chain, Address, H256, List, List2D, Mapping, StorageFlags, U256};
use parity_scale_codec::Encode;

#[test]
fn off_chain_engine_storage_and_caller() {
    off_chain::with_engine(|e| {
        e.set_caller([1u8; 20]);
        e.set_call_data(&[]);
    });
    let caller = env().caller();
    assert_eq!(caller, Address::from([1u8; 20]));

    env().set_storage(StorageFlags::empty(), b"key", b"value");
    off_chain::with_engine(|e| {
        let v = e.get_storage_value(b"key").unwrap();
        assert_eq!(v, b"value");
    });
}

/// Off-chain Env: call_data_size, call_data_copy, call_data_load. 覆盖 call_data 相关分支。
#[test]
fn off_chain_env_call_data() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_call_data(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    });
    assert_eq!(env().call_data_size(), 10);
    let copied = env().call_data_copy(0, 5);
    assert_eq!(copied, vec![1, 2, 3, 4, 5]);
    let copied_tail = env().call_data_copy(8, 5);
    assert_eq!(copied_tail, vec![9, 10]);
    let load = env().call_data_load(0);
    assert_eq!(load[0..10], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    assert_eq!(load[10..], [0u8; 22]);
}

/// Off-chain Env: empty call_data. 空 call_data 时 call_data_copy 返回零填充。
#[test]
fn off_chain_env_call_data_empty() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_call_data(&[]);
    });
    assert_eq!(env().call_data_size(), 0);
    let copied = env().call_data_copy(0, 4);
    assert_eq!(copied, vec![0, 0, 0, 0]);
    let load = env().call_data_load(0);
    assert_eq!(load, [0u8; 32]);
}

/// Off-chain Env: address, balance, balance_of, chain_id, gas_price, base_fee, origin, now, gas_limit, value_transferred, return_data_size.
#[test]
fn off_chain_env_read_only_stubs() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([5u8; 20]);
    });
    let addr = env().address();
    assert_eq!(addr, Address::zero());
    let balance = env().balance();
    assert_eq!(balance, U256::ZERO);
    let balance_of = env().balance_of(&[1u8; 20]);
    assert_eq!(balance_of, U256::ZERO);
    let chain_id = env().chain_id();
    assert_eq!(chain_id[31], 1);
    assert_eq!(env().gas_price(), 1);
    let base_fee = env().base_fee();
    assert_eq!(base_fee, U256::ZERO);
    assert_eq!(env().origin(), [5u8; 20]);
    let now = env().now();
    assert!(now > 0);
    assert_eq!(env().gas_limit(), u64::MAX);
    assert_eq!(env().value_transferred(), U256::ZERO);
    assert_eq!(env().return_data_size(), 0);
}

/// Off-chain Env: code_hash, code_size. 链下返回零。
#[test]
fn off_chain_env_code() {
    off_chain::with_engine(|e| e.reset());
    assert_eq!(env().code_hash(&[0u8; 20]), H256::zero());
    assert_eq!(env().code_size(&[0u8; 20]), 0);
}

/// Off-chain Env: set_storage_or_clear (zero clears), get_storage_or_zero. 零值清除存储；get 不到返回零。
#[test]
fn off_chain_env_storage_or_clear_and_zero() {
    off_chain::with_engine(|e| e.reset());
    let key = [1u8; 32];
    let value = [2u8; 32];
    env().set_storage(StorageFlags::empty(), &key, &value);
    let got = env().get_storage_or_zero(StorageFlags::empty(), &key);
    assert_eq!(got, value);
    let zero = [0u8; 32];
    let prev = env().set_storage_or_clear(StorageFlags::empty(), &key, &zero);
    assert_eq!(prev, Some(32));
    let after = env().get_storage_or_zero(StorageFlags::empty(), &key);
    assert_eq!(after, [0u8; 32]);
    let missing = env().get_storage_or_zero(StorageFlags::empty(), &[9u8; 32]);
    assert_eq!(missing, [0u8; 32]);
}

/// Off-chain Env: set_storage_or_clear with non-zero writes. 非零值写入。
#[test]
fn off_chain_env_storage_or_clear_non_zero() {
    off_chain::with_engine(|e| e.reset());
    let key = [3u8; 32];
    let value = [4u8; 32];
    env().set_storage_or_clear(StorageFlags::empty(), &key, &value);
    let got = env().get_storage_or_zero(StorageFlags::empty(), &key);
    assert_eq!(got, value);
}

/// Off-chain Env: hash_keccak_256 (when feature off_chain). Keccak-256 哈希。
#[test]
fn off_chain_env_hash_keccak_256() {
    off_chain::with_engine(|e| e.reset());
    let hash = env().hash_keccak_256(b"hello");
    assert_eq!(hash.as_bytes().len(), 32);
    let hash2 = env().hash_keccak_256(b"hello");
    assert_eq!(hash, hash2);
}

/// Off-chain Env: call and delegate_call return error. 链下 call/delegate_call 返回错误。
#[test]
fn off_chain_env_call_returns_err() {
    off_chain::with_engine(|e| e.reset());
    use pallet_revive_uapi::CallFlags;
    let callee = Address::zero();
    let deposit = U256::ZERO;
    let value = U256::ZERO;
    let r = env().call(
        CallFlags::empty(),
        &callee,
        0,
        0,
        &deposit,
        &value,
        &[],
        None,
    );
    assert!(r.is_err());
    let r2 = env().delegate_call(
        CallFlags::empty(),
        &callee,
        0,
        0,
        &deposit,
        &[],
        None,
    );
    assert!(r2.is_err());
}

/// Off-chain Env: instantiate fills address and returns error. 链下 instantiate 填 address 并返回错误。
#[test]
fn off_chain_env_instantiate() {
    off_chain::with_engine(|e| e.reset());
    use pallet_revive_uapi::CallFlags;
    let mut address = [0xffu8; 20];
    let code_hash = [0u8; 32];
    let deposit = [0u8; 32];
    let value = [0u8; 32];
    let r = env().instantiate(
        CallFlags::empty(),
        &code_hash,
        0,
        0,
        &deposit,
        &value,
        &[],
        &mut address,
        None,
    );
    // Off-chain instantiate now succeeds and assigns a mock address (for multi-contract tests).
    assert!(r.is_ok());
    assert_ne!(address[16..20], [0u8; 4]); // at least id bytes set
}

/// Off-chain Env: get_immutable_data / set_immutable_data no-op. 链下不可变数据为空操作。
#[test]
fn off_chain_env_immutable_data() {
    off_chain::with_engine(|e| e.reset());
    let mut buf = [0u8; 8];
    let mut cursor: &mut [u8] = &mut buf;
    env().get_immutable_data(&mut cursor);
    env().set_immutable_data(&[]);
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

/// Mapping get for non-existent key returns Err. 不存在的 key 返回 Err。
#[test]
fn mapping_get_nonexistent_returns_err() {
    off_chain::with_engine(|e| e.reset());
    let m: Mapping<[u8; 20], u64> = Mapping::new(b"balance_nx");
    let alice = [1u8; 20];
    let e = env();
    let r = m.get(e, &alice);
    assert!(r.is_err());
}

/// Mapping set overwrites; get returns latest. 重复 set 后 get 为最后一次写入。
#[test]
fn mapping_set_overwrite() {
    off_chain::with_engine(|e| e.reset());
    let m: Mapping<u32, u64> = Mapping::new(b"cnt_ov");
    let e = env();
    m.set(e, &1u32, &100u64).unwrap();
    assert_eq!(m.get(e, &1u32).unwrap(), 100);
    m.set(e, &1u32, &200u64).unwrap();
    assert_eq!(m.get(e, &1u32).unwrap(), 200);
}

/// Mapping clear removes key; get after clear returns Err. set 后 clear，再 get 应得到 Err。
#[test]
fn mapping_clear_then_get_err() {
    off_chain::with_engine(|e| e.reset());
    let m: Mapping<u32, u64> = Mapping::new(b"cnt_del");
    let e = env();
    m.set(e, &1u32, &100u64).unwrap();
    assert_eq!(m.get(e, &1u32).unwrap(), 100);
    m.clear(e, &1u32).unwrap();
    let r = m.get(e, &1u32);
    assert!(r.is_err());
}

/// Storage clear removes key; get after clear returns Err. set 后 clear，再 get 应得到 Err。
#[test]
fn storage_clear_then_get_err() {
    off_chain::with_engine(|e| e.reset());
    let s: super::Storage<u64> = super::Storage::new(b"st_del");
    let e = env();
    s.set(e, &123u64);
    assert_eq!(s.get(e).unwrap(), 123u64);
    s.clear(e);
    let r = s.get(e);
    assert!(r.is_err());
}

/// Mapping full_key with buf too small returns None. full_key 的 buf 不足时返回 None。
#[test]
fn mapping_full_key_buf_too_small() {
    let m: Mapping<u32, u64> = Mapping::new(b"pfx");
    let key_bytes = 1u32.encode();
    let mut buf = [0u8; 2];
    let full = m.full_key(&key_bytes, &mut buf);
    assert!(full.is_none());
    let mut buf_ok = [0u8; 8];
    let full_ok = m.full_key(&key_bytes, &mut buf_ok);
    assert!(full_ok.is_some());
    assert_eq!(full_ok.unwrap().len(), m.prefix().len() + key_bytes.len());
}

/// Mapping set_bytes / get_bytes roundtrip. 按字节 set/get 往返一致。
#[test]
fn mapping_set_bytes_get_bytes() {
    off_chain::with_engine(|e| e.reset());
    let m: Mapping<u32, u64> = Mapping::new(b"bytes_m");
    let e = env();
    let key = 42u32;
    let key_bytes = key.encode();
    let value_bytes = 100u64.encode();
    let mut buf = vec![0u8; m.prefix().len() + key_bytes.len()];
    m.set_bytes(e, &key_bytes, &mut buf, &value_bytes).unwrap();
    let mut read_buf = vec![0u8; m.prefix().len() + key_bytes.len()];
    let out = m.get_bytes(e, &key_bytes, &mut read_buf).unwrap();
    assert_eq!(out, value_bytes);
    let decoded: u64 = parity_scale_codec::Decode::decode(&mut &out[..]).unwrap();
    assert_eq!(decoded, 100);
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
fn list_u32_clear() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u32, u64> = List::new(b"list_cl_next", b"list_cl_items");
    let e = env();
    assert_eq!(list.insert(e, &10u64).unwrap(), 0);
    assert_eq!(list.insert(e, &20u64).unwrap(), 1);
    assert_eq!(list.len(e), 2);
    assert!(list.contains(e, &0));
    assert!(list.contains(e, &1));

    list.clear(e, &0).unwrap();
    assert!(!list.contains(e, &0));
    assert!(list.get(e, &0).is_none());
    assert_eq!(list.len(e), 2, "clear should not change next_id/len");

    let page = list.list(e, 0, 2);
    assert_eq!(page, vec![(1, 20)]);
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

/// List desc_list on empty list returns empty. 空列表降序分页返回空。
#[test]
fn list_u32_desc_list_empty() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u32, u64> = List::new(b"list_t5e_next", b"list_t5e_items");
    let e = env();
    let empty = list.desc_list(e, None, 5);
    assert!(empty.is_empty());
    let empty2 = list.desc_list(e, Some(0), 1);
    assert!(empty2.is_empty());
}

/// List get for out-of-range key returns None. 越界 id 返回 None。
#[test]
fn list_u32_get_out_of_range() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u32, u64> = List::new(b"list_t5r_next", b"list_t5r_items");
    let e = env();
    list.insert(e, &1u64).unwrap();
    assert!(list.get(e, &1).is_none());
    assert!(list.get(e, &100).is_none());
}

/// List insert when u8 id would overflow returns None. u8 作为 id 时溢出 insert 返回 None。
#[test]
fn list_u8_insert_overflow_returns_none() {
    off_chain::with_engine(|e| e.reset());
    let list: List<u8, u32> = List::new(b"list_u8o_next", b"list_u8o_items");
    let e = env();
    for i in 0..255 {
        let k = list.insert(e, &(i as u32));
        assert!(k.is_some(), "insert {} should succeed", i);
    }
    assert_eq!(list.len(e), 255u8);
    let k_next = list.insert(e, &999u32);
    assert!(k_next.is_none(), "u8 overflow (256th insert) should return None");
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
fn list_2d_clear() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_cl_k1", b"dl_cl_len", b"dl_cl_k2", b"dl_cl_store");
    let e = env();
    let alice = [1u8; 20];
    assert_eq!(dl.insert(e, &alice, &10u64).unwrap(), 0);
    assert_eq!(dl.insert(e, &alice, &20u64).unwrap(), 1);
    assert_eq!(dl.len(e, &alice), 2);
    assert_eq!(dl.get(e, &alice, 0).unwrap(), 10);
    assert_eq!(dl.get(e, &alice, 1).unwrap(), 20);

    dl.clear(e, &alice, 0).unwrap();
    assert!(dl.get(e, &alice, 0).is_none());
    assert_eq!(dl.len(e, &alice), 2, "clear should not change next_id/len");

    let page = dl.list(e, &alice, 0, 2);
    assert_eq!(page, vec![(1, 20)]);
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

/// List2D get for non-existent k1 returns None. 不存在的 k1 返回 None。
#[test]
fn list_2d_get_nonexistent_k1() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_gn1_k1", b"dl_gn1_len", b"dl_gn1_k2", b"dl_gn1_store");
    let e = env();
    let alice = [1u8; 20];
    assert!(dl.get(e, &alice, 0).is_none());
    assert!(dl.len(e, &alice) == 0u32);
}

/// List2D get for valid k1 but invalid k2 returns None. 存在的 k1、不存在的 k2 返回 None。
#[test]
fn list_2d_get_valid_k1_invalid_k2() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_gv1_k1", b"dl_gv1_len", b"dl_gv1_k2", b"dl_gv1_store");
    let e = env();
    let alice = [1u8; 20];
    dl.insert(e, &alice, &100u64).unwrap();
    assert!(dl.get(e, &alice, 1).is_none());
    assert!(dl.get(e, &alice, 100).is_none());
}

/// List2D update for non-existent k1 returns None. 对不存在的 k1  update 返回 None。
#[test]
fn list_2d_update_nonexistent_k1() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_un1_k1", b"dl_un1_len", b"dl_un1_k2", b"dl_un1_store");
    let e = env();
    let alice = [1u8; 20];
    let r = dl.update(e, &alice, 0, &999u64);
    assert!(r.is_none());
}

/// List2D desc_list with size 0 returns empty. 降序分页 size=0 返回空。
#[test]
fn list_2d_desc_list_size_zero() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_ds0_k1", b"dl_ds0_len", b"dl_ds0_k2", b"dl_ds0_store");
    let e = env();
    let alice = [1u8; 20];
    dl.insert(e, &alice, &1u64).unwrap();
    let empty = dl.desc_list(e, &alice, None, 0);
    assert!(empty.is_empty());
}

/// List2D list_all for non-existent k1 returns empty. 不存在的 k1  list_all 返回空。
#[test]
fn list_2d_list_all_nonexistent_k1() {
    off_chain::with_engine(|e| e.reset());
    let dl: List2D<[u8; 20], u32, u64> =
        List2D::new(b"dl_la0_k1", b"dl_la0_len", b"dl_la0_k2", b"dl_la0_store");
    let e = env();
    let alice = [1u8; 20];
    let empty = dl.list_all(e, &alice);
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