//! 对应 ink fallible-setter 的 it_works（成功路径；revert 时 off_chain 会 panic，链上会正确 revert）

use crate::fallible_setter;
use wrevive_api::off_chain;

#[test]
fn it_works() {
    off_chain::with_engine(|e| {
        e.reset();
        e.set_caller([0u8; 20]);
    });
    fallible_setter::deploy();
    assert_eq!(fallible_setter::get(), 0);

    fallible_setter::try_set(1);
    assert_eq!(fallible_setter::get(), 1);

    fallible_setter::try_set(2);
    assert_eq!(fallible_setter::get(), 2);

    // 链上：try_set(2) 再调会 NoChange revert；try_set(101) 会 TooLarge revert
    // off_chain 下 REVERT 会 panic，此处仅测成功路径
}
