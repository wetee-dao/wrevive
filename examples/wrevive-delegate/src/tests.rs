//! 单元测试：wrevive-delegate 代理调用目标合约。
//! 使用 off_chain Engine 注册“假”目标（收到 get_value selector 即返回 100）与 delegate，
//! 通过 delegate 的 fallback 将 get_value 的 call_data 转发到 target，断言返回 100。

use blake2::Digest;
use std::panic;
use wrevive_api::off_chain::{ReturnValuePanic, with_engine};
use wrevive_api::{Address, Decode, Encode, Env, ReturnFlags, env};

/// 与 wrevive-macro 一致：Blake2s256(name)[0..4] 作为 message selector。
fn selector_from_name(name: &str) -> [u8; 4] {
    let hash = blake2::Blake2s256::digest(name.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
}

