//! 单元测试：wrevive-delegate 代理调用目标合约。
//! 使用 off_chain Engine 注册“假”目标（收到 get_value selector 即返回 100）与 delegate，
//! 通过 delegate 的 fallback 将 get_value 的 call_data 转发到 target，断言返回 100。
