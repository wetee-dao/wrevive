# wrevive - 基于 pallet-revive 的 Rust 合约工具链与示例

本仓库是一个 **Cargo workspace**，包含：

- **`wrevive-api`**：合约运行时 API（链上/链下统一 `Env`、Storage/Mapping/List/List2D 等）。
- **`wrevive-macro`**：ink!-style 过程宏：`#[revive_contract]` + `#[revive(constructor)]`/`#[revive(message)]`，自动生成 `deploy()`/`call()` 分发与 ABI。
- **`cargo-wrevive`**：`cargo wrevive build` 子命令：把合约构建为 PolkaVM `.polkavm`，并生成 ABI 文件。
- **examples**：两份示例合约：
  - `wrevive-codec-contract`：使用 `wrevive-api` + `wrevive-macro`（推荐写法）。
  - `wrevive-sol-contract`：使用 `pvm_contract_macros` 的 Solidity 风格示例（更贴近 EVM 思路，依赖链上环境，host 下可能无法直接 `cargo test`/`cargo check`）。

> 说明：本文档以 **Linux + Rust stable** 为例，命令均在仓库根目录运行。

## 快速开始（复制即用）

### 前置要求

- **Rust 工具链（推荐 nightly）**：`cargo wrevive build` 会使用 `-Z ...` 等不稳定参数。
  - 若你只想用 stable：需要配合 `RUSTC_BOOTSTRAP=1`（不推荐用于生产环境，只建议本地构建/试验）。
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
# 构建 wrevive-codec-contract（推荐示例）
cargo wrevive build -p wrevive-codec-contract
```

构建产物（workspace 根 `target/` 下）：

- **PolkaVM 字节码**：`target/<bin>.release.polkavm`（例如 `target/wrevive-codec.release.polkavm`）
- **ABI（JSON）**：`target/<bin>.release.abi.json`（由 cargo-wrevive 生成，偏 EVM 风格）
- **ABI（ink 风格）**：`target/contract/<contract_name>.json`（由 `#[revive_contract]` 编译期生成；你也会看到类似 `target/wrevive-codec.json` 的文件）

> 具体文件名取决于 bin name 与宏内的 contract_name 推导，建议 `ls target/` 查看。

### 运行单元测试（链下 off_chain）

```bash
# wrevive-api 单元测试（包含 off_chain Env 覆盖）
cargo test -p wrevive-api

# 示例合约单元测试（off_chain Engine）
cargo test -p wrevive-codec-contract
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
│   ├── wrevive-codec-contract/
│   └── wrevive-sol-contract/
├── COVERAGE.md
└── README.md
```

## 合约写法（推荐：wrevive-api + wrevive-macro）

以 `examples/wrevive-codec-contract/src/contract.rs` 为例：

- **入口生成**：在 `mod contract { ... }` 上加 `#[revive_contract]`
- **构造函数**：`#[revive(constructor)] pub fn deploy() ...`
- **消息函数**：`#[revive(message)] pub fn foo(...) ...`
- **存储工具**：
  - `storage!(b"...")` → `Storage<T>`
  - `mapping!(b"...")` → `Mapping<K,V>`
  - `list!(b"...")` → `List<Idx, V>`
  - `list_2d!(b"...")` → `List2D<K1, Idx, V>`

> `storage!/mapping!/list!/list_2d!` 的 prefix 会通过 Blake2s256 生成 4 字节前缀，并在 `#[revive_contract]` 展开时做重复检查。

## 调用约定（selector + SCALE 参数）

`#[revive_contract]` 生成的 `call()` 会：

1. 读取 call data **前 4 字节**作为 selector（大端 `u32::from_be_bytes`）。
2. 将 `call data[4..]` 作为 **SCALE 编码**的参数流，按函数参数顺序逐个 `Decode`。

因此，编码规则是：

- `payload = selector(4 bytes) ++ SCALE(args...)`

selector 生成规则：

- 若写了 `#[revive(message, selector = 0x...)]`：用你指定的 4 字节 selector
- 否则：使用 **BLAKE2s256(function_name) 的前 4 字节**（与 ink! 一致）


## 常见问题

### 为什么 `wrevive-sol-contract` 在 host 下可能编不过？

`wrevive-sol-contract` 使用 `pvm_contract_macros` + `pallet_revive_uapi::HostFnImpl` 的链上接口，很多方法需要特定目标/运行时环境。建议把它当作“链上风格示例”，日常开发与测试优先使用 `wrevive-codec-contract`（off_chain 可直接单测）。

## 参考

- [pallet-revive 源码](https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame/revive)
- [pallet-revive-uapi 文档](https://paritytech.github.io/polkadot-sdk/master/pallet_revive_uapi/)
- [PolkaVM](https://github.com/paritytech/polkavm)
