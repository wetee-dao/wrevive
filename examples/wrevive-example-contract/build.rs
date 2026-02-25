fn main() {
    // 仅在为 PolkaVM 目标编译时运行 PVM 构建；test 时用 host target，不跑此构建器
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("polkavm") {
        cargo_pvm_contract_builder::PvmBuilder::new().build();
    }
}
