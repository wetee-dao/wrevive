//! 对应 ink erc721：mint, balance_of, owner_of, transfer

use crate::erc721;
use wrevive_api::off_chain;

fn alice() -> [u8; 20] {
    [1u8; 20]
}
fn bob() -> [u8; 20] {
    [2u8; 20]
}

#[test]
fn mint_and_owner_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice());
    });
    erc721::deploy();
    assert_eq!(erc721::balance_of(alice()), 0);
    assert_eq!(erc721::owner_of(1), [0u8; 20]);

    erc721::mint(1);
    assert_eq!(erc721::owner_of(1), alice());
    assert_eq!(erc721::balance_of(alice()), 1);

    erc721::mint(2);
    assert_eq!(erc721::balance_of(alice()), 2);
}

#[test]
fn transfer_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller(alice());
    });
    erc721::deploy();
    erc721::mint(1);
    erc721::transfer(alice(), bob(), 1);
    assert_eq!(erc721::owner_of(1), bob());
    assert_eq!(erc721::balance_of(alice()), 0);
    assert_eq!(erc721::balance_of(bob()), 1);
}
