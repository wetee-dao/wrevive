# Wetee Contract - 基于 pallet-revive 的 Rust 智能合约

这是一个直接使用 `pallet-revive-uapi` 编写的 Rust 智能合约示例，不依赖 ink! 框架。提供类似 ink! 的 **`#[ink(constructor)]`** 与 **`#[ink(message)]`** 宏，由宏生成 `deploy()`/`call()` 与 selector 分发，无需手写 match。

## 项目结构

```
wetee-contract-core/   # 仓库名可保留，内部 crates 为 wrevive-*
├── Cargo.toml                              # Rust 项目配置
├── build.sh                                # 构建脚本
├── crates/
│   └── wrevive-macro/                      # #[revive(constructor)] / #[revive(message)] 过程宏
├── src/
│   └── lib.rs                              # 合约（#[revive_contract] mod + #[revive(...)]）
├── abi/                                    # ABI 规范与生成结果
├── rust-contract.md                        # 详细开发文档
└── README.md                               # 本文件
```

## 快速开始

### 前置要求

1. **Rust 工具链** (stable 版本)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **polkatool** (用于将 ELF 链接为 PolkaVM 字节码；`polkavm-linker` 仅为库，无独立二进制)
   ```bash
   cargo install polkatool
   ```

3. **Rust 标准库源码** (用于交叉编译)
   ```bash
   rustup component add rust-src
   ```

### 构建合约

使用提供的构建脚本：

```bash
./build.sh
```

或者手动构建：

```bash
# 编译
RUSTC_BOOTSTRAP=1 cargo +stable build \
  --release \
  --no-default-features \
  --target riscv64emac-unknown-none-polkavm.json \
  -Z build-std=core,alloc

# 链接（需要先安装 polkatool）
polkatool link --strip -o target/contract/contract.polkavm \
  target/riscv64emac-unknown-none-polkavm/release/example.elf
```

### 构建输出

- **ELF 文件**: `target/riscv64emac-unknown-none-polkavm/release/example.elf`
- **PolkaVM 文件**: `target/contract/contract.polkavm` (链接后)
- **ABI**: `abi/contract.json`（类似 ink!，供前端/JS 编码调用与解码返回值）

## 宏用法（类似 ink!）

在模块上标注 `#[revive_contract]`，模块内用 **`#[revive(constructor)]`** 标记部署函数、用 **`#[revive(message, selector = 0x...)]`** 标记可调用的 message，宏会自动生成 `deploy()`/`call()` 及按 selector 的分发逻辑：

```rust
use wrevive_macro::{revive, revive_contract};
use wrevive_api::{ext, ReturnFlags, StorageFlags};

#[revive_contract]
mod contract {
    #[ink(constructor)]
    pub fn deploy() { ... }

    #[ink(message, selector = 0x60fe47b1)]
    pub fn set_value(value: u32) { ... }

    #[ink(message, selector = 0x6d4ce633)]
    pub fn get_value() -> u32 { ... }
}
```

支持的参数/返回类型：`u32`、`[u8; 20]`（AccountId）、`()`。其他类型可在 `crates/wrevive-macro` 中扩展。

## 合约功能

当前示例合约实现了以下功能：

1. **存储值管理**
   - `set(uint32)`: 设置一个 u32 值
   - `get()`: 获取存储的值

2. **Owner 管理**
   - `setOwner(address)`: 设置合约所有者（仅当前 owner 可调用）
   - `getOwner()`: 获取当前所有者地址

## ABI 与 JS 调用

合约提供类似 ink! 的 **ABI 描述**，便于前端/JS 按接口编码调用并解码返回值：

- **ABI 文件**: [`abi/contract.json`](abi/contract.json)  
  **由 `#[revive_contract]` 宏在编译时**从 `#[ink(constructor)]` / `#[ink(message)]` 自动生成，无需手写 spec 或注释块。详见 [abi/README.md](abi/README.md)。
- **调用约定**: 单入口 `call()`，payload = **selector (4 字节)** + 参数（u32 小端 4 字节，AccountId 20 字节）。
- **JS 示例**: [`examples/call-with-abi.js`](examples/call-with-abi.js)  
  使用 `@polkadot/api`，演示如何根据 ABI 编码 `set`/`get`/`setOwner`/`getOwner` 并解码返回。

| 方法     | Selector (hex) | 参数        | 返回   |
|----------|----------------|-------------|--------|
| set      | 0x60fe47b1     | value: u32  | -      |
| get      | 0x6d4ce633     | -           | u32    |
| setOwner | 0x13af4035     | owner: 20B  | -      |
| getOwner | 0x8f8f9f8f     | -           | 20B    |

3. **事件**
   - 当值被设置时发送事件

## 部署合约

### 使用 Substrate 客户端

```rust
use pallet_revive::Pallet;

// 读取合约字节码
let contract_code = std::fs::read("target/contract/contract.polkavm")?;

// 上传合约代码
let upload_result = Pallet::<T>::bare_upload_code(
    origin,
    contract_code,
    None, // storage_deposit_limit
)?;

let code_hash = upload_result.code_hash;

// 实例化合约
let instantiate_result = Pallet::<T>::bare_instantiate(
    origin,
    code_hash,
    0u64.into(), // value
    Weight::MAX, // gas_limit
    None,        // storage_deposit_limit
    Vec::new(),  // data (构造函数参数)
    true,        // bump_nonce
)?;

let contract_address = instantiate_result.account_id;
```

### 使用 Polkadot.js API

```javascript
const contractCode = fs.readFileSync('target/contract/contract.polkavm');

// 上传代码
const uploadTx = api.tx.revive.uploadCode(contractCode, null);
await uploadTx.signAndSend(signer, ({ status, events }) => {
    if (status.isInBlock) {
        console.log('合约代码已上传');
        // 从事件中提取 code_hash
    }
});

// 实例化合约
const instantiateTx = api.tx.revive.instantiate(
    0,           // value
    Weight.MAX,  // gas_limit
    null,         // storage_deposit_limit
    codeHash,     // code_hash
    [],           // data
    true          // salt
);
await instantiateTx.signAndSend(signer);
```

## 调用合约

### 编码调用数据

合约使用函数选择器来分发调用。示例：

```rust
// set(uint32) 的调用数据
let value: u32 = 42;
let mut call_data = Vec::new();
call_data.extend_from_slice(&[0x60, 0xfe, 0x47, 0xb1]); // selector
call_data.extend_from_slice(&value.to_le_bytes());      // 参数
```

### 调用合约方法

```rust
let call_result = Pallet::<T>::bare_call(
    origin,
    contract_address,
    0u64.into(),        // value
    Weight::MAX,        // gas_limit
    None,               // storage_deposit_limit
    call_data,          // 编码的调用数据
    ExecConfig {
        bump_nonce: true,
        collect_deposit_from_hold: None,
        effective_gas_price: None,
    },
)?;
```

## 开发指南

详细的开发文档请参考 [rust-contract.md](./rust-contract.md)，包含：

- 完整的 API 参考
- 存储操作示例
- 合约间调用示例
- 事件发送示例
- 错误处理指南
- 与 ink! 的对比

## 注意事项

1. **内存管理**: 合约运行在 `no_std` 环境，使用静态缓冲区
2. **编码格式**: 调用数据可以使用 SCALE 或 Solidity ABI 编码
3. **Gas/Weight**: 使用 Substrate Weight 模型，需要指定 `ref_time` 和 `proof_size`
4. **调试**: 开发环境可以使用 `ext::debug`，生产环境应移除

## 故障排除

### 编译错误

如果遇到编译错误，检查：

1. Rust 版本是否为 stable
2. 是否设置了 `RUSTC_BOOTSTRAP=1`
3. 是否安装了 `rust-src` 组件
4. 目标 JSON 文件路径是否正确

### 链接错误

如果 `polkatool` 未找到：

```bash
cargo install polkatool
```

### 运行时错误

- 检查合约字节码是否正确上传
- 验证调用数据编码是否正确
- 确认 gas_limit 足够

## 参考资源

- [pallet-revive 源码](https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame/revive)
- [pallet-revive-uapi 文档](https://paritytech.github.io/polkadot-sdk/master/pallet_revive_uapi/)
- [PolkaVM 项目](https://github.com/paritytech/polkavm)
- [Polkadot SDK 文档](https://docs.substrate.io/)

## 许可证

本项目遵循项目根目录的许可证。
