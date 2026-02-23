use super::{env, off_chain, Mapping, StorageFlags};

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
    let m = Mapping::new(b"balance");
    let alice = [1u8; 20];
    let bob = [2u8; 20];
    let subkey: u32 = 0;
    let e = env();
    m.set(e, &alice, &subkey, &1000u64).unwrap();
    m.set(e, &bob, &subkey, &2000u64).unwrap();

    let v: u64 = m.get(e, &alice, &subkey).unwrap();
    assert_eq!(v, 1000);
    let v2: u64 = m.get(e, &bob, &subkey).unwrap();
    assert_eq!(v2, 2000);

    let val42: &[u8] = b"val42";
    let subkey_bytes: &[u8] = b"sub";
    m.set(e, &42u32, &subkey_bytes, val42).unwrap();
    let out: Vec<u8> = m.get(e, &42u32, &subkey_bytes).unwrap();
    assert_eq!(&out[..], b"val42");

    // key 支持基础类型（u32, u64 等）
    let m2 = Mapping::new(b"cnt");
    let subkey2: u8 = 0;
    m2.set(e, &1u32, &subkey2, &100u64).unwrap();
    m2.set(e, &2u32, &subkey2, &200u64).unwrap();
    assert_eq!(m2.get::<_, _, u64>(e, &1u32, &subkey2).unwrap(), 100);
    assert_eq!(m2.get::<_, _, u64>(e, &2u32, &subkey2).unwrap(), 200);
}