# 测试覆盖率 / Test Coverage

## 如何生成 / How to generate

安装 [cargo-tarpaulin](https://github.com/cargo-bins/cargo-tarpaulin) 后，在仓库根目录执行：

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --exclude wrevive-sol-contract --out Stdout
```

生成 HTML 报告（输出到 `coverage/` 目录）：

```bash
cargo tarpaulin --workspace --exclude wrevive-sol-contract --out Html --output-dir coverage
```

然后在浏览器中打开 `coverage/tarpaulin-report.html` 查看逐文件覆盖率。

## 当前覆盖率概览 / Current coverage summary

（由 `cargo tarpaulin --workspace --exclude wrevive-sol-contract --out Stdout` 生成）

| 包/文件 | 覆盖行数 | 总行数 | 说明 |
|--------|----------|--------|------|
| **wrevive-api** (核心 API) | | | |
| list.rs | 48 | 52 | 高，List 逻辑基本覆盖 |
| list_2d.rs | 82 | 87 | 高，List2D 逻辑基本覆盖 |
| mapping.rs | 31 | 33 | 高，含 full_key/set_bytes/get_bytes 等 |
| off_chain.rs | **109** | **109** | **100%**，Env 各方法均有单测 |
| on_chain.rs | 0 | 95 | 未覆盖，单元测试使用 off_chain 后端 |
| lib.rs | 7 | 15 | 部分，Storage/reexport 等被间接使用 |
| **cargo-wrevive** | 0 | ~349 | 无单元测试，由集成/手动使用覆盖 |
| **wrevive-macro** | 0 | 377 | 过程宏，由编译示例合约间接覆盖，tarpaulin 不统计宏展开代码 |

**整体：约 25% 行覆盖率**（277/1117）。  
**wrevive-api 中可被单元测试覆盖的部分（不含 on_chain）：约 94%**（277/296 行）。on_chain 仅在链上编译时使用，不参与 host 单元测试。

## 是否需要更多测试以达到 80%？/ Do we need more tests for 80%?

- **若目标为「wrevive-api 可覆盖代码达到 80%」**：已达成（当前 ~94%），无需再为 API 加测。
- **若目标为「整个 workspace 行覆盖 80%」**：需要为 **cargo-wrevive** 和/或 **wrevive-macro** 增加可被统计的测试：
  - **cargo-wrevive**：可为 `abi.rs`（如 `parse_sol_params`、`extract_sol_path_from_source`）、`lib.rs`（如 `get_bin_targets`、`get_target_root`）等抽成可测函数并写单元测试；CLI 入口 `main.rs` 适合用集成测试或脚本测。
  - **wrevive-macro**：过程宏代码由 tarpaulin 统计的是宏 crate 自身；若要在数字上提高覆盖率，可考虑用 [trybuild](https://github.com/dtolnay/trybuild) 做编译测试，或增加依赖该宏的集成测试（覆盖率仍可能归到调用方）。
  - **on_chain**：设计上不在 host 单元测试中运行，不纳入「可覆盖」目标即可。

结论：**核心库 wrevive-api 的单元测试覆盖率已足够高；若希望全仓库数值达到 80%，需要重点为 cargo-wrevive 补充单元/集成测试。**

## 说明 / Notes

- **on_chain**：仅在链上或非 test 编译时使用，单元测试统一走 **off_chain**，故 on_chain 行覆盖为 0 属预期。
- **cargo-wrevive**：CLI 工具，适合通过集成测试或脚本测试，可后续补充。
- **wrevive-macro**：过程宏在编译时展开，覆盖率工具通常只统计过程宏 crate 自身代码，不统计展开后的合约代码；合约逻辑由示例测试间接覆盖。
