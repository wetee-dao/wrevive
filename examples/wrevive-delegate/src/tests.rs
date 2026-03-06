//! 单元测试：wrevive-delegate 代理调用目标合约。
//! 使用 off_chain Engine 注册“假”目标（收到 get_value selector 即返回 100）与 delegate，
//! 通过 delegate 的 fallback 将 get_value 的 call_data 转发到 target，断言返回 100。

use std::panic;
use blake2::Digest;
use wrevive_api::off_chain::{with_engine, ReturnValuePanic};
use wrevive_api::{Address, Decode, Encode, ReturnFlags, env};

/// 与 wrevive-macro 一致：Blake2s256(name)[0..4] 作为 message selector。
fn selector_from_name(name: &str) -> [u8; 4] {
    let hash = blake2::Blake2s256::digest(name.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
}

/// 通过 delegate 代理调用 target 的 get_value：先注册“假” target（call 时直接 return 100）、
/// 部署 delegate(target_addr)，再向 delegate 发送 get_value 的 call_data，
/// fallback 会 delegate_call 到 target，得到 100。
#[test]
fn delegate_proxies_get_value_to_contract() {
    let target_addr = Address::from([1u8; 20]);
    let delegate_addr = Address::from([2u8; 20]);
    let get_value_selector = selector_from_name("get_value");

    with_engine(|e| {
        e.reset_all();
        e.register_contract(target_addr, move || {
            let api = env();
            let len = api.call_data_size() as usize;
            let data = api.call_data_copy(0, len);
            let sel: [u8; 4] = if data.len() >= 4 {
                data[0..4].try_into().unwrap()
            } else {
                [0u8; 4]
            };
            if sel == get_value_selector {
                let ret = 100u32.encode();
                api.return_value(ReturnFlags::empty(), &ret);
            }
        });
        e.register_contract(delegate_addr, || crate::call());
    });

    // 部署 delegate，保存 target 地址（deploy 入口会 return_value，需 catch_unwind）
    with_engine(|e| {
        e.set_contract(delegate_addr);
        e.set_caller(Address::from([0u8; 20]).into());
        e.set_call_data(&target_addr.encode());
    });
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| crate::deploy()));

    // 通过 delegate 调用 get_value：call_data = selector(get_value)，走 fallback -> delegate_call(target)
    let call_data = get_value_selector.to_vec();
    with_engine(|e| {
        e.set_contract(delegate_addr);
        e.set_caller(Address::from([0u8; 20]).into());
        e.set_call_data(&call_data);
    });

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| crate::call()));
    match result {
        Ok(()) => panic!("contract::call() 应通过 return_value panic 返回，不应正常返回"),
        Err(e) => {
            let panic = e
                .downcast_ref::<ReturnValuePanic>()
                .unwrap_or_else(|| panic!("应为 ReturnValuePanic，实际: {:?}", e));
            let value: u32 =
                Decode::decode(&mut &panic.1[..]).unwrap_or_else(|_| panic!("解码 u32 失败，长度 {}", panic.1.len()));
            assert_eq!(value, 100, "通过 delegate 代理应得到 target 返回的 100");
        }
    }
}
