//! 生成 polkavm 编译目标；CONTRACT_LIB_NAME 供 #[revive_contract] 宏生成 ABI 合约名。
fn main() {
    let _ = polkavm_linker::target_json_path(polkavm_linker::TargetJsonArgs::default());
    let contract_lib_name = contract_lib_name_from_manifest();
    println!("cargo:rustc-env=CONTRACT_LIB_NAME={}", contract_lib_name);
}

fn contract_lib_name_from_manifest() -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "contract".into());
    let toml_path = std::path::Path::new(&manifest_dir).join("Cargo.toml");
    let content = match std::fs::read_to_string(&toml_path) {
        Ok(c) => c,
        Err(_) => return pkg_name.replace('-', "_"),
    };
    let mut in_lib = false;
    for line in content.lines() {
        let line = line.trim();
        if line == "[lib]" {
            in_lib = true;
            continue;
        }
        if in_lib {
            if line.starts_with('[') {
                break;
            }
            if line.starts_with("name") {
                if let Some(q) = line.strip_prefix("name").and_then(|s| s.trim().strip_prefix('=')) {
                    let q = q.trim().trim_matches(|c| c == '"' || c == '\'');
                    if !q.is_empty() {
                        return q.to_string();
                    }
                }
            }
        }
    }
    pkg_name.replace('-', "_")
}
