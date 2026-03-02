//! 从 Cargo.toml 读取 [[bin]] name 等（ABI 文件名用）。

use std::env;
use std::fs;
use std::path::Path;

/// 从当前包的 Cargo.toml 中读取第一个 `[[bin]]` 的 `name`，用于 ABI 文件名。
pub fn bin_name_from_manifest() -> Option<String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").ok()?;
    let content = fs::read_to_string(Path::new(&manifest_dir).join("Cargo.toml")).ok()?;
    let rest = content.find("[[bin]]")?;
    let after_bin = &content[rest + "[[bin]]".len()..];
    let name_start = after_bin.find("name")?;
    let after_name = &after_bin[name_start + 4..];
    let eq = after_name.find('=')?;
    let after_eq = after_name[eq + 1..].trim_start();
    let quote = after_eq.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = after_eq[1..].find(quote)?;
    Some(after_eq[1..1 + end].to_string())
}
