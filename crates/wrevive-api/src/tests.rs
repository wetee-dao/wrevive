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