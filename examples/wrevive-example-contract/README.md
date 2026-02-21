# wrevive-example-contract

示例 PolkaVM 合约：value/owner 存储，提供 `deploy`、`set_value`、`get_value`、`set_owner`、`get_owner`。

## 构建（链上）

从仓库根目录：

```bash
./build.sh
# 或指定包名: ./build.sh wrevive-example-contract
```

或直接使用 cargo：

```bash
cargo build -p wrevive-example-contract --release --no-default-features --target riscv64emac-unknown-none-polkavm -Z build-std=core
```

（需先运行一次构建以生成 target json，或从 wrevive-example-contract 目录执行 `cargo build` 触发 build.rs）

## 单元测试（wrevive-api off_chain）

依赖 `wrevive-api` 的 **off_chain** 能力：启用 feature `test` 后，合约在 host 上使用内存 Engine 而非链上 Host，可直接调用 `contract::deploy()`、`contract::set_value()` 等并断言 storage/events。

从仓库根目录运行：

```bash
cargo test -p wrevive-example-contract --features test
```

测试中通过 `wrevive_api::off_chain::with_engine(|e| { ... })` 配置 caller、call_data，并用 `e.get_storage_value(key)`、`e.take_events()` 做断言。
