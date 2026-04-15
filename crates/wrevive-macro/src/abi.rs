//! ink! 风格 ABI JSON 生成与写入 target/contract/{name}.json。

use crate::type_reg::TypeReg;
use crate::docs;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use syn::{FnArg, Fields, GenericParam, Item, ItemFn, Pat, ReturnType, Type};

    /// Replaces `-` with `_` in contract names to avoid invalid Go identifiers in go-ink-gen.
    /// 合约名中 `-` 改为 `_`，避免 go-ink-gen 生成非法 Go 标识符（expected ';', found '-'）。
    /// 
    /// # English
    /// Converts contract names to Go-safe identifiers by replacing hyphens with underscores.
    /// Prevents "expected ';', found '-'" errors in generated Go code.
    /// 
    /// # 中文
    /// 通过将连字符替换为下划线将合约名称转换为 Go 安全标识符。
    /// 防止生成的 Go 代码中出现 "expected ';', found '-'" 错误。
fn contract_name_go_safe(name: &str) -> String {
    name.replace('-', "_")
}

    /// Formats a 4-byte selector as a hex string for the ABI selector field.
    /// 将 4 字节 selector 格式化为十六进制字符串，用于 ABI 的 selector 字段。
    /// 
    /// # English
    /// Converts a 4-byte selector array into a hex string with "0x" prefix.
    /// Used in ABI JSON generation for function selectors.
    /// 
    /// # 中文
    /// 将 4 字节选择器数组转换为带 "0x" 前缀的十六进制字符串。
    /// 用于 ABI JSON 生成中的函数选择器。
pub fn selector_hex(sel: [u8; 4]) -> String {
    format!("0x{:02x}{:02x}{:02x}{:02x}", sel[0], sel[1], sel[2], sel[3])
}

    /// Extracts (struct_name, fields) from an Item or single struct definition within a mod.
    /// 从 mod 内 Item 或单个 struct 定义中提取 (struct_name, fields)。
    /// 
    /// # English
    /// Parses a struct definition and returns its name and named fields with their types.
    /// Only works for structs with named fields.
    /// 
    /// # 中文
    /// 解析结构体定义并返回其名称和带有类型的命名字段。
    /// 仅适用于具有命名字段的结构体。
fn struct_fields_from_item(item: &Item) -> Option<(String, Vec<(String, Type)>)> {
    let Item::Struct(s) = item else { return None };
    let syn::Fields::Named(named) = &s.fields else { return None };
    let fields: Vec<(String, Type)> = named
        .named
        .iter()
        .filter_map(|f| {
            let name = f.ident.as_ref().map(|i| i.to_string())?;
            Some((name, f.ty.clone()))
        })
        .collect();
    if fields.is_empty() {
        return None;
    }
    Some((s.ident.to_string(), fields))
}

    /// Extracts (enum_name, list of variant names) from an Item or single enum definition within a mod; only returns if all variants are unit (no fields).
    /// 从 mod 内 Item 或单个 enum 定义中提取 (enum_name, 变体名列表)，仅当所有变体无字段时返回。
    /// 
    /// # English
    /// Parses an enum and returns its name and the names of all variants.
    /// Only works for C-like enums where all variants have no fields.
    /// 
    /// # 中文
    /// 解析枚举并返回其名称和所有变体的名称。
    /// 仅适用于所有变体都没有字段的类 C 枚举。
fn enum_variants_from_item(item: &Item) -> Option<(String, Vec<String>)> {
    let Item::Enum(e) = item else { return None };
    let mut names = Vec::new();
    for v in &e.variants {
        if !matches!(v.fields, Fields::Unit) {
            return None;
        }
        names.push(v.ident.to_string());
    }
    if names.is_empty() {
        return None;
    }
    Some((e.ident.to_string(), names))
}

    /// Extracts (enum_name, variants_with_fields) from enum definition; each variant is (variant_name, list of field types).
    /// Used to generate ABI variants with fields (e.g., AssetInfo: Native(Bytes), ERC20(Bytes, H256)).
    /// Does not handle generic enums (e.g., EditType<T>), which are collected by generic_enum_from_item.
    /// 从 enum 定义中提取 (enum_name, variants_with_fields)，每个变体为 (变体名, 字段类型列表)。
    /// 用于生成 ABI 中带 fields 的 variant（如 AssetInfo: Native(Bytes), ERC20(Bytes, H256)）。
    /// 若 enum 带泛型（如 EditType<T>），不在此处理，由 generic_enum_from_item 收集。
    /// 
    /// # English
    /// Parses an enum with variants that may have fields.
    /// Returns the enum name and each variant with its field types.
    /// Does not handle generic enums; those are processed separately.
    /// 
    /// # 中文
    /// 解析可能带有字段的变体的枚举。
    /// 返回枚举名称和每个带有其字段类型的变体。
    /// 不处理泛型枚举；那些会单独处理。
fn enum_variants_with_fields_from_item(item: &Item) -> Option<(String, Vec<(String, Vec<Type>)>)> {
    let Item::Enum(e) = item else { return None };
    if !e.generics.params.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for v in &e.variants {
        let field_tys: Vec<Type> = match &v.fields {
            Fields::Unit => vec![],
            Fields::Unnamed(u) => u.unnamed.iter().map(|f| f.ty.clone()).collect(),
            Fields::Named(n) => n.named.iter().map(|f| f.ty.clone()).collect(),
        };
        out.push((v.ident.to_string(), field_tys));
    }
    if out.is_empty() {
        return None;
    }
    Some((e.ident.to_string(), out))
}

/// 从带泛型的 enum（如 EditType<T>）提取 (enum_name, (param_names, variants))，用于实例化时推断类型生成 ABI。
fn generic_enum_from_item(item: &Item) -> Option<(String, (Vec<String>, Vec<(String, Vec<Type>)>))> {
    let Item::Enum(e) = item else { return None };
    let params: Vec<String> = e
        .generics
        .params
        .iter()
        .filter_map(|p| {
            if let GenericParam::Type(t) = p {
                Some(t.ident.to_string())
            } else {
                None
            }
        })
        .collect();
    if params.is_empty() {
        return None;
    }
    let mut variants = Vec::new();
    for v in &e.variants {
        let field_tys: Vec<Type> = match &v.fields {
            Fields::Unit => vec![],
            Fields::Unnamed(u) => u.unnamed.iter().map(|f| f.ty.clone()).collect(),
            Fields::Named(n) => n.named.iter().map(|f| f.ty.clone()).collect(),
        };
        variants.push((v.ident.to_string(), field_tys));
    }
    if variants.is_empty() {
        return None;
    }
    Some((e.ident.to_string(), (params, variants)))
}

/// 从 `pub type Alias = Underlying` 提取 (alias_name, underlying_ty)。
fn type_alias_from_item(item: &Item) -> Option<(String, Type)> {
    let Item::Type(t) = item else { return None };
    Some((t.ident.to_string(), (*t.ty).clone()))
}

/// 从合约 mod 的 other_items 与 crate 源码（含 path 依赖）收集 struct/enum/type 定义，
/// struct 用于 ABI composite 的 fields，enum 用于 ABI variant 的 variants（含带字段枚举），type 用于 ABI 类型别名（如 NodeID = u64）。
fn collect_struct_defs(
    manifest_dir: &Path,
    mod_items: &[Item],
) -> (
    HashMap<String, Vec<(String, Type)>>,
    HashMap<String, Vec<String>>,
    HashMap<String, Type>,
    HashMap<String, Vec<(String, Vec<Type>)>>,
    HashMap<String, (Vec<String>, Vec<(String, Vec<Type>)>)>,
) {
    let mut struct_defs = HashMap::<String, Vec<(String, Type)>>::new();
    let mut enum_defs = HashMap::<String, Vec<String>>::new();
    let mut type_aliases = HashMap::<String, Type>::new();
    let mut enum_defs_with_fields = HashMap::<String, Vec<(String, Vec<Type>)>>::new();
    let mut generic_enum_defs = HashMap::<String, (Vec<String>, Vec<(String, Vec<Type>)>)>::new();
    for item in mod_items {
        if let Some((name, fields)) = struct_fields_from_item(item) {
            struct_defs.entry(name).or_insert(fields);
        }
        if let Some((name, vs)) = enum_variants_from_item(item) {
            enum_defs.entry(name).or_insert(vs);
        }
        if let Some((name, variants)) = enum_variants_with_fields_from_item(item) {
            // 与 enum_defs 一致用 or_insert：同名枚举后出现的依赖 crate 不得覆盖本 crate（如 Cloud 的 Error 被 subnet 覆盖）。
            enum_defs_with_fields.entry(name).or_insert(variants);
        }
        if let Some((name, generic_def)) = generic_enum_from_item(item) {
            generic_enum_defs.entry(name).or_insert(generic_def);
        }
        if let Some((name, ty)) = type_alias_from_item(item) {
            type_aliases.insert(name, ty);
        }
    }
    let src_dir = manifest_dir.join("src");
    if src_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "rs") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(file) = syn::parse_file(&content) {
                            for item in file.items {
                                if let Some((name, fields)) = struct_fields_from_item(&item) {
                                    struct_defs.entry(name).or_insert(fields);
                                }
                                if let Some((name, vs)) = enum_variants_from_item(&item) {
                                    enum_defs.entry(name).or_insert(vs);
                                }
                                if let Some((name, variants)) = enum_variants_with_fields_from_item(&item) {
                                    enum_defs_with_fields.entry(name).or_insert(variants);
                                }
                                if let Some((name, generic_def)) = generic_enum_from_item(&item) {
                                    generic_enum_defs.entry(name).or_insert(generic_def);
                                }
                                if let Some((name, ty)) = type_alias_from_item(&item) {
                                    type_aliases.insert(name, ty);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if let Ok(cargo) = fs::read_to_string(manifest_dir.join("Cargo.toml")) {
        for line in cargo.lines() {
            let line = line.trim();
            if let Some(idx) = line.find("path") {
                let rest = line.get(idx + 4..).unwrap_or("").trim_start();
                if rest.starts_with('=') {
                    let rest = rest[1..].trim_start();
                    let path_str = rest
                        .trim_start_matches('"')
                        .split('"')
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !path_str.is_empty() && !path_str.starts_with("git") {
                        let dep_src = manifest_dir.join(path_str).join("src");
                        if dep_src.is_dir() {
                            if let Ok(entries) = fs::read_dir(&dep_src) {
                                for entry in entries.flatten() {
                                    let path = entry.path();
                                    if path.extension().map_or(false, |e| e == "rs") {
                                        if let Ok(content) = fs::read_to_string(&path) {
                                            if let Ok(file) = syn::parse_file(&content) {
                                                for item in file.items {
                                                    if let Some((name, fields)) = struct_fields_from_item(&item) {
                                                        struct_defs.entry(name).or_insert(fields);
                                                    }
                                                    if let Some((name, vs)) = enum_variants_from_item(&item) {
                                                        enum_defs.entry(name).or_insert(vs);
                                                    }
                                                    if let Some((name, variants)) = enum_variants_with_fields_from_item(&item) {
                                                        enum_defs_with_fields.entry(name).or_insert(variants);
                                                    }
                                                    if let Some((name, generic_def)) = generic_enum_from_item(&item) {
                                                        generic_enum_defs.entry(name).or_insert(generic_def);
                                                    }
                                                    if let Some((name, ty)) = type_alias_from_item(&item) {
                                                        type_aliases.insert(name, ty);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    (struct_defs, enum_defs, type_aliases, enum_defs_with_fields, generic_enum_defs)
}

/// 根据合约名、构造函数与 message 列表生成 ink! 风格 ABI JSON。
/// 输出路径：以当前项目根或 workspace 根为基准，写入 `target/contract/{name}.json`。
/// `mod_items`：合约 mod 内除 constructor/message 外的项，用于收集同 mod 内 struct 定义以生成 composite fields。
/// 返回 `Ok(())` 表示成功，`Err(msg)` 表示失败（调用方负责打印结果）。
pub fn emit_abi(
    contract_name: &str,
    constructor_fn: &ItemFn,
    message_fns: &[(ItemFn, [u8; 4], bool)],
    mod_items: &[Item],
) -> Result<(), String> {
    if env::var("CARGO_BIN_NAME").is_err() {
        return Ok(());
    }
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "CARGO_MANIFEST_DIR 未设置".to_string())?;
    let manifest_path = Path::new(&manifest_dir);
    let manifest_abs = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        env::current_dir()
            .ok()
            .map(|cwd| cwd.join(manifest_path))
            .unwrap_or_else(|| manifest_path.to_path_buf())
    };

    fn find_workspace_root(start: &Path) -> Option<PathBuf> {
        let mut current = start;
        loop {
            let workspace_toml = current.join("Cargo.toml");
            if workspace_toml.exists() {
                if let Ok(content) = fs::read_to_string(&workspace_toml) {
                    if content.contains("[workspace]") {
                        return Some(current.to_path_buf());
                    }
                }
            }
            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }
        None
    }

    // 基准目录：优先 workspace 根，否则当前项目（包）根
    let base_dir = find_workspace_root(&manifest_abs).unwrap_or_else(|| manifest_abs.clone());
    let out_dir = base_dir.join("target");
    fs::create_dir_all(&out_dir).map_err(|e| {
        format!("创建目录 {} 失败: {}", out_dir.display(), e)
    })?;
    let out_path = out_dir.join(format!("{}.json", contract_name));

    let mut reg = TypeReg::new();
    let (struct_defs, enum_defs, type_aliases, enum_defs_with_fields, generic_enum_defs) =
        collect_struct_defs(&manifest_abs, mod_items);
    reg.register_struct_defs(struct_defs);
    reg.register_enum_defs(enum_defs);
    reg.register_type_aliases(type_aliases);
    reg.register_enum_defs_with_fields(enum_defs_with_fields);
    reg.register_generic_enum_defs(generic_enum_defs);

    // 提取构造函数文档注释
    let constructor_docs = docs::clean_docs(&docs::extract_docs(&constructor_fn.attrs));
    let constructor_param_docs = docs::parse_param_docs(&constructor_docs);

    let mut constructor_args = Vec::new();
    for arg in &constructor_fn.sig.inputs {
        let FnArg::Typed(pt) = arg else { continue };
        let arg_name = match pt.pat.as_ref() {
            Pat::Ident(pi) => pi.ident.to_string(),
            _ => continue,
        };
        if let Some((type_id, display_name)) = reg.ensure_type(pt.ty.as_ref()) {
            let param_doc = constructor_param_docs.get(&arg_name)
                .map(|doc| vec![doc.clone()])
                .unwrap_or_else(Vec::new);
            constructor_args.push(serde_json::json!({
                "label": arg_name,
                "type": { "displayName": display_name, "type": type_id },
                "docs": param_doc
            }));
        }
    }
    let constructor_return = match &constructor_fn.sig.output {
        ReturnType::Default => {
            let (type_id, display_name) = reg.ensure_constructor_result();
            serde_json::json!({ "displayName": display_name, "type": type_id })
        }
        ReturnType::Type(_, ty) => {
            let (type_id, display_name) = reg
                .ensure_type(ty)
                .unwrap_or((0, vec!["ScaleBytes".into()]));
            serde_json::json!({ "displayName": display_name, "type": type_id })
        }
    };
    let constructors = vec![serde_json::json!({
        "args": constructor_args,
        "default": false,
        "docs": constructor_docs,
        "label": constructor_fn.sig.ident.to_string(),
        "payable": false,
        "returnType": constructor_return,
        "selector": "0x00000000"
    })];

    let mut messages = Vec::new();
    for (f, sel, explicit_mutates) in message_fns {
        let label = f.sig.ident.to_string();
        // mutates 仅来自 #[revive(message, write)] 显式标记
        let mutates = *explicit_mutates;
        let selector = selector_hex(*sel);
        
        // 提取消息函数文档注释
        let message_docs = docs::clean_docs(&docs::extract_docs(&f.attrs));
        let message_param_docs = docs::parse_param_docs(&message_docs);
        
        let mut args = Vec::new();
        for arg in &f.sig.inputs {
            let FnArg::Typed(pt) = arg else { continue };
            let arg_name = match pt.pat.as_ref() {
                Pat::Ident(pi) => pi.ident.to_string(),
                _ => continue,
            };
            if let Some((type_id, display_name)) = reg.ensure_type(pt.ty.as_ref()) {
                let param_doc = message_param_docs.get(&arg_name)
                    .map(|doc| vec![doc.clone()])
                    .unwrap_or_else(Vec::new);
                args.push(serde_json::json!({
                    "label": arg_name,
                    "type": { "displayName": display_name, "type": type_id },
                    "docs": param_doc
                }));
            }
        }
        let return_type = match &f.sig.output {
            ReturnType::Default => {
                let (type_id, display_name) = reg.ensure_message_result_unit();
                serde_json::json!({ "displayName": display_name, "type": type_id })
            }
            // 所有返回值在外层都包一层 Result<_, LangError>；若声明为 Result<(), Error> 则为两层 Result。
            ReturnType::Type(_, ty) => {
                let (type_id, display_name) = reg
                    .ensure_message_result(ty)
                    .unwrap_or_else(|| reg.ensure_type(ty).unwrap_or((0, vec!["ScaleBytes".into()])));
                serde_json::json!({ "displayName": display_name, "type": type_id })
            }
        };
        messages.push(serde_json::json!({
            "args": args,
            "default": false,
            "docs": message_docs,
            "label": label,
            "mutates": mutates,
            "payable": false,
            "returnType": return_type,
            "selector": selector
        }));
    }

    let lang_error_id = reg.ensure_lang_error();
    let lang_error = serde_json::json!({
        "displayName": ["ink", "LangError"],
        "type": lang_error_id
    });

    // Frontend no longer treats AccountId specially; model it as raw 32-byte array.
    // This keeps ABI environment consistent with "accountId = bytes32".
    let id_account = syn::parse_str::<syn::Type>("[u8; 32]")
        .ok()
        .and_then(|t| reg.ensure_type(&t).map(|(id, _)| id))
        .unwrap_or(0);
    let id_balance = syn::parse_str::<syn::Type>("u128")
        .ok()
        .and_then(|t| reg.ensure_type(&t).map(|(id, _)| id))
        .unwrap_or(0);
    let id_block = syn::parse_str::<syn::Type>("u32")
        .ok()
        .and_then(|t| reg.ensure_type(&t).map(|(id, _)| id))
        .unwrap_or(0);
    let id_ts = syn::parse_str::<syn::Type>("u64")
        .ok()
        .and_then(|t| reg.ensure_type(&t).map(|(id, _)| id))
        .unwrap_or(0);
    let id_hash = reg.ensure_hash();
    let environment = serde_json::json!({
        "accountId": { "displayName": ["[u8; 32]"], "type": id_account },
        "balance": { "displayName": ["Balance"], "type": id_balance },
        "blockNumber": { "displayName": ["BlockNumber"], "type": id_block },
        "hash": { "displayName": ["Hash"], "type": id_hash },
        "nativeToEthRatio": 100000000,
        "staticBufferSize": 16384,
        "timestamp": { "displayName": ["Timestamp"], "type": id_ts }
    });

    let types_array: Vec<serde_json::Value> = reg.types_array().to_vec();
    let name_go = contract_name_go_safe(contract_name);

    let spec_obj = serde_json::json!({
        "constructors": constructors,
        "docs": [],
        "lang_error": lang_error,
        "events": [],
        "messages": messages,
        "environment": environment
    });
    let abi = serde_json::json!({
        "source": {
            "hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "language": "wrevive",
            "compiler": "rustc"
        },
        "contract": {
            "name": name_go,
            "version": "0.1.0"
        },
        "image": null,
        "spec": spec_obj,
        "storage": null,
        "types": types_array,
        "version": 6
    });

    let json_str = serde_json::to_string_pretty(&abi).map_err(|e| {
        format!("序列化 ABI JSON 失败: {}", e)
    })?;
    fs::write(&out_path, &json_str).map_err(|e| {
        format!("写入 ABI 文件失败 {}: {}", out_path.display(), e)
    })?;
    Ok(())
}
