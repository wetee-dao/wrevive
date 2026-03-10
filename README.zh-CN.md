# wrevive — 封装 `pallet-revive`（PolkaVM）的 Rust 合约工具链

[English](README.md)

> **状态：开发中，暂不可用于生产环境。**  
> API / ABI / 行为可能随时变更，请自行评估风险。

**wrevive** 是对 **pallet-revive 的封装**，为 PolkaVM 合约开发提供 **类似 ink! 的开发体验**。它抽象了底层的 pallet-revive UAPI 调用，同时保持完全兼容。

## wrevive 是什么？

- **定位**：封装 pallet-revive 的底层 UAPI，提供开发者友好的类似 ink! 的体验
- **核心特性**：
  - 声明式存储宏：`storage!`、`mapping!`、`list!`、`list_2d!`
  - 自动入口生成：`#[revive_contract]` 自动生成 `deploy()`/`call()`
  - 统一的链上/链下测试：同一 `Env` trait 适用于两种环境
  - ABI 生成：类似 ink! 的 JSON ABI，便于工具集成
- **兼容性**：编译为原生的 pallet-revive 合约 - 无运行时开销

本仓库是一个 **Cargo workspace**，包含：

- **`wrevive-api`**：合约运行时 API（链上/链下统一 `Env` 和高层存储抽象：`Storage`/`Mapping`/`List`/`List2D`）
- **`wrevive-macro`**：ink! 风格过程宏：`#[revive_contract]` + `#[revive(constructor)]`/`#[revive(message)]`，自动生成 `deploy()`/`call()` 分发与 ABI
- **`cargo-wrevive`**：`cargo wrevive build` 子命令：把合约构建为 PolkaVM `.polkavm`，并生成 ABI 文件
- **examples**：
  - `wrevive-contract`：使用 `wrevive-api` + `wrevive-macro` 的 SCALE(codec) 示例合约（推荐）

> 说明：本文档以 **Linux** 为例，命令均在仓库根目录运行。

## 快速开始（复制即用）

### 前置要求

- **Rust 工具链（推荐 nightly）**：`cargo wrevive build` 可能会使用 `-Z ...` 等不稳定参数。
  - 若你只想用 stable：可能需要配合 `RUSTC_BOOTSTRAP=1`（不推荐用于生产环境，只建议本地构建/试验）。
- **rust-src**（交叉编译 build-std 需要）

```bash
rustup component add rust-src
```

### 安装 `cargo wrevive`

```bash
cargo install --path crates/cargo-wrevive
```

### 构建示例合约（生成 `.polkavm` + ABI）

```bash
cargo wrevive build -p wrevive-contract
```

构建产物（workspace 根 `target/` 下）：

- **PolkaVM 字节码**：`target/<bin>.release.polkavm`
- **ABI（JSON）**：`target/<bin>.release.abi.json`（由 `cargo-wrevive` 生成，偏 EVM 风格）
- **ABI（ink 风格）**：`target/contract/<contract_name>.json`（由 `#[revive_contract]` 编译期生成）

> 具体文件名取决于 bin name 与宏内 contract_name 的推导，建议构建后 `ls target/` 查看。

### 运行单元测试（链下 off_chain）

```bash
# wrevive-api 单元测试（包含 off_chain Env）
cargo test -p wrevive-api

# 示例合约单元测试（off_chain Engine）
cargo test -p wrevive-contract
```

## 项目结构（当前 workspace）

```
wrevive/
├── Cargo.toml
├── crates/
│   ├── wrevive-api/
│   ├── wrevive-macro/
│   └── cargo-wrevive/
├── examples/
│   └── wrevive-contract/
├── COVERAGE.md
├── README.md
└── README.zh-CN.md
```

## 合约写法（推荐：wrevive-api + wrevive-macro）

以 `examples/wrevive-contract/src/contract.rs` 为完整示例：

### 基本合约结构

```rust
#[revive_contract]
pub mod contract {
    use wrevive_api::Address;
    use wrevive_macro::{storage, mapping, list, list_2d, revive_contract};
    
    // 存储定义
    const VALUE: Storage<u32> = storage!(b"value");
    const BALANCE: Mapping<Address, u64> = mapping!(b"balance");
    const RECORDS: List<u32, u64> = list!(b"records");
    
    // 构造函数（必须恰好一个）
    #[revive(constructor)]
    pub fn deploy(initial_value: u32) -> Result<(), Error> {
        VALUE.set(&initial_value);
        Ok(())
    }
    
    // 消息函数
    #[revive(message, write)]
    pub fn set_value(value: u32) -> Result<(), Error> {
        VALUE.set(&value);
        Ok(())
    }
    
    #[revive(message)]
    pub fn get_value() -> u32 {
        VALUE.get().unwrap_or(0)
    }
}
```

### 存储工具宏

- **`storage!(b"...")`** → `Storage<T>`：单值存储
- **`mapping!(b"...")`** → `Mapping<K, V>`：键值映射存储
- **`list!(b"...")`** → `List<Idx, V>`：一维列表（自增索引）
- **`list_2d!(b"...")`** → `List2D<K1, Idx, V>`：二维列表（外键 + 内部自增）

> `storage!/mapping!/list!/list_2d!` 的 prefix 会通过 Blake2s256 生成前 4 字节，并在 `#[revive_contract]` 展开时做重复检查。

### 消息属性

- **`#[revive(message)]`**：只读消息（纯视图）
- **`#[revive(message, write)]`**：修改存储（需要修改状态的函数）
- **`#[revive(message, sol)]`**：使用 Solidity 编码而非 SCALE
- **`#[revive(message, selector = 0x12345678)]`**：自定义 4 字节选择器
- **`#[revive(constructor)]`**：合约构造函数（必须恰好一个）
- **`#[revive(fallback)]`**：未知选择器的回退函数（可选，最多一个）

### 返回值类型

所有合约函数（构造函数和消息函数）必须有返回值：
- **`Result<T, E>`**：可能失败的操作（推荐）
- **`T`**：总是成功的操作
- **`Option<T>`**：可能返回无值的操作

示例错误类型：
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum Error {
    InsufficientBalance,
    Unauthorized,
    NotFound,
}
```

### 常见数据类型（wrevive-api）

`wrevive-api` 提供可直接用于存储/消息的 SCALE 可编码类型，可从 `wrevive_api` 引入：

| 类型 | 说明 | 编码 |
|------|------|------|
| `Address` | 20 字节地址（EVM/账户兼容） | 20 字节 |
| `H256` | 32 字节哈希 | 32 字节 |
| `U256` | 256 位无符号整数（大端，EVM 兼容） | 32 字节 |
| `BlockNumber` | 区块高度（`u32` 类型别名） | 同 u32 |
| `Bytes` | 变长字节（`Vec<u8>` 别名） | 长度前缀 + 字节 |

示例：`Storage<Address>`、`Mapping<Address, U256>`、`Mapping<H256, Bytes>` 等；`Address` / `H256` 与 `[u8;20]` / `[u8;32]` 可互转（`From` / `Into`）。

## 调用约定（selector + SCALE 参数）

`#[revive_contract]` 生成的 `call()` 会：

1. 读取 call data **前 4 字节**作为 selector（大端 `u32::from_be_bytes`）。
2. 将 `call_data[4..]` 作为 **SCALE 编码**的参数流，按函数参数顺序逐个 `Decode`。

因此，编码规则是：

- `payload = selector(4 bytes) ++ SCALE(args...)`

selector 生成规则：

- 若写了 `#[revive(message, selector = 0x...)]`：用你指定的 4 字节 selector
- 否则：使用 **BLAKE2s256(function_name) 的前 4 字节**（与 ink! 一致）

## 参考

- [pallet-revive 源码](https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame/revive)
- [pallet-revive-uapi 文档](https://paritytech.github.io/polkadot-sdk/master/pallet_revive_uapi/)
- [PolkaVM](https://github.com/paritytech/polkavm)

