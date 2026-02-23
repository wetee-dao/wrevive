//! 对应 ink dns：register, set_address, get_address, get_owner

use crate::dns;
use wrevive_api::off_chain;

fn alice() -> [u8; 20] {
    [1u8; 20]
}
fn bob() -> [u8; 20] {
    [2u8; 20]
}

fn name_hash(s: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let n = s.len().min(32);
    out[..n].copy_from_slice(&s[..n]);
    out
}

#[test]
fn register_and_get_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice());
    });
    dns::deploy();
    let name = name_hash(b"polkadot");
    assert_eq!(dns::get_owner(name), [0u8; 20]);
    dns::register(name);
    assert_eq!(dns::get_owner(name), alice());
    assert_eq!(dns::get_address(name), alice());
}

#[test]
fn set_address_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice());
    });
    dns::deploy();
    let name = name_hash(b"eth");
    dns::register(name);
    dns::set_address(name, bob());
    assert_eq!(dns::get_address(name), bob());
    assert_eq!(dns::get_owner(name), alice());
}
