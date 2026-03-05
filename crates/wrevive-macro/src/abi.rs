//! ink! 风格 ABI JSON 生成与写入 target/contract/{name}.json。

use crate::type_reg::TypeReg;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use syn::{Block, Expr, FnArg, Fields, GenericParam, ImplItem, Item, ItemFn, Pat, ReturnType, Stmt, Type};

/// 合约名中 `-` 改为 `_`，避免 go-ink-gen 生成非法 Go 标识符（expected ';', found '-'）。
fn contract_name_go_safe(name: &str) -> String {
    name.replace('-', "_")
}

/// 将 4 字节 selector 格式化为十六进制字符串，用于 ABI 的 selector 字段。
pub fn selector_hex(sel: [u8; 4]) -> String {
    format!("0x{:02x}{:02x}{:02x}{:02x}", sel[0], sel[1], sel[2], sel[3])
}

/// 仅当 receiver 为 env() 时计为写操作的 env 方法（deposit_event 不算写操作，不在此列）。
const ENV_WRITE_METHODS: &[&str] = &[
    "set_storage",
    "clear_storage",
    "set_storage_or_clear",
    "transfer",
    "call",
    "delegate_call",
    "call_evm",
    "delegate_call_evm",
    "terminate",
];

/// 存储抽象（Storage/Mapping/List 等）的写方法；无类型信息时仅能按方法名判断。
const STORAGE_WRITE_METHODS: &[&str] = &["set", "insert", "remove", "clear"];

/// receiver 是否为对 env 的调用（env() / crate::env() / wrevive_api::env() 等）。
fn receiver_is_env_call(receiver: &Expr) -> bool {
    let Expr::Call(c) = receiver else { return false };
    let Expr::Path(p) = &*c.func else { return false };
    p.path
        .segments
        .last()
        .map(|s| s.ident == "env")
        .unwrap_or(false)
}

fn is_env_write_method(name: &str) -> bool {
    ENV_WRITE_METHODS.contains(&name)
}

fn is_storage_write_method(name: &str) -> bool {
    STORAGE_WRITE_METHODS.contains(&name)
}

/// 从调用表达式中的 func 提取被调函数名（仅当 func 为简单路径时，如 my_fn 或 mod::my_fn）。
fn callee_name_from_expr(expr: &Expr) -> Option<String> {
    if let Expr::Path(p) = expr {
        return Some(crate::attrs::path_to_display_name(&p.path));
    }
    None
}

/// 递归检查表达式中是否包含修改状态的调用；若传入 callee_mutates，则被调用的函数若会修改状态也视为 mutates。
fn expr_contains_mutating_call_with<F: Fn(&str) -> bool>(expr: &Expr, callee_mutates: &F) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            let name = m.method.to_string();
            // env 写操作：仅当 receiver 为 env() 时计为写（set_storage/clear_storage/transfer/call/...，不含 deposit_event）
            if is_env_write_method(&name) && receiver_is_env_call(&m.receiver) {
                return true;
            }
            if is_storage_write_method(&name) {
                return true;
            }
            if expr_contains_mutating_call_with(&m.receiver, callee_mutates) {
                return true;
            }
            for arg in &m.args {
                if expr_contains_mutating_call_with(arg, callee_mutates) {
                    return true;
                }
            }
            false
        }
        Expr::Block(b) => block_contains_mutating_call_with(&b.block, callee_mutates),
        Expr::If(e) => {
            expr_contains_mutating_call_with(&e.cond, callee_mutates)
                || block_contains_mutating_call_with(&e.then_branch, callee_mutates)
                || e.else_branch
                    .as_ref()
                    .map(|(_, e)| expr_contains_mutating_call_with(e, callee_mutates))
                    .unwrap_or(false)
        }
        Expr::Match(e) => {
            if expr_contains_mutating_call_with(&e.expr, callee_mutates) {
                return true;
            }
            for arm in &e.arms {
                if expr_contains_mutating_call_with(&arm.body, callee_mutates) {
                    return true;
                }
            }
            false
        }
        Expr::Call(c) => {
            if let Some(name) = callee_name_from_expr(&c.func) {
                if callee_mutates(&name) {
                    return true;
                }
            }
            if expr_contains_mutating_call_with(&c.func, callee_mutates) {
                return true;
            }
            for arg in &c.args {
                if expr_contains_mutating_call_with(arg, callee_mutates) {
                    return true;
                }
            }
            false
        }
        Expr::Let(e) => expr_contains_mutating_call_with(&e.expr, callee_mutates),
        Expr::Return(e) => e
            .expr
            .as_ref()
            .map_or(false, |e| expr_contains_mutating_call_with(e, callee_mutates)),
        Expr::Assign(a) => {
            expr_contains_mutating_call_with(&a.left, callee_mutates)
                || expr_contains_mutating_call_with(&a.right, callee_mutates)
        }
        Expr::Closure(c) => expr_contains_mutating_call_with(&c.body, callee_mutates),
        Expr::Loop(l) => block_contains_mutating_call_with(&l.body, callee_mutates),
        Expr::While(w) => {
            expr_contains_mutating_call_with(&w.cond, callee_mutates)
                || block_contains_mutating_call_with(&w.body, callee_mutates)
        }
        Expr::ForLoop(f) => {
            expr_contains_mutating_call_with(&f.expr, callee_mutates)
                || block_contains_mutating_call_with(&f.body, callee_mutates)
        }
        Expr::Unsafe(u) => block_contains_mutating_call_with(&u.block, callee_mutates),
        Expr::Async(a) => block_contains_mutating_call_with(&a.block, callee_mutates),
        Expr::TryBlock(t) => block_contains_mutating_call_with(&t.block, callee_mutates),
        Expr::Try(t) => expr_contains_mutating_call_with(&t.expr, callee_mutates),
        _ => false,
    }
}

fn block_contains_mutating_call_with<F: Fn(&str) -> bool>(block: &Block, callee_mutates: &F) -> bool {
    for stmt in &block.stmts {
        if stmt_contains_mutating_call_with(stmt, callee_mutates) {
            return true;
        }
    }
    false
}

fn stmt_contains_mutating_call_with<F: Fn(&str) -> bool>(stmt: &Stmt, callee_mutates: &F) -> bool {
    match stmt {
        Stmt::Expr(expr, _) => expr_contains_mutating_call_with(expr, callee_mutates),
        Stmt::Local(local) => local
            .init
            .as_ref()
            .map(|init| expr_contains_mutating_call_with(&init.expr, callee_mutates))
            .unwrap_or(false),
        Stmt::Item(_) | Stmt::Macro(_) => false,
    }
}

/// 收集合约 mod 内所有可调用函数（constructor、messages、mod_items 中的 fn 与 impl 方法），用于传递闭包分析 mutates。
fn collect_all_blocks<'a>(
    constructor_fn: &'a ItemFn,
    message_fns: &'a [(ItemFn, [u8; 4])],
    mod_items: &'a [Item],
) -> Vec<(String, &'a Block)> {
    let mut out = Vec::new();
    out.push((constructor_fn.sig.ident.to_string(), &*constructor_fn.block));
    for (f, _) in message_fns {
        out.push((f.sig.ident.to_string(), &*f.block));
    }
    for item in mod_items {
        if let Item::Fn(f) = item {
            out.push((f.sig.ident.to_string(), &*f.block));
        }
        if let Item::Impl(i) = item {
            for impl_item in &i.items {
                if let ImplItem::Fn(f) = impl_item {
                    out.push((f.sig.ident.to_string(), &f.block));
                }
            }
        }
    }
    out
}

/// mutates: true 的核心规则：合约函数是否在链上执行了写操作。
/// 写操作 = env 上（set_storage/clear_storage/set_storage_or_clear/transfer/call/delegate_call/call_evm/delegate_call_evm/terminate）或存储（set/insert/remove/clear）
/// 或存储抽象（set/insert/remove/clear）。若函数自身或它调用的任意函数中存在上述调用，则 mutates: true。
/// 固定点计算：每个函数是否（直接或通过调用的函数）涉及上述写操作；用于 ABI mutates 标记。
fn compute_mutates_fixpoint(
    constructor_fn: &ItemFn,
    message_fns: &[(ItemFn, [u8; 4])],
    mod_items: &[Item],
) -> HashMap<String, bool> {
    let all_fns = collect_all_blocks(constructor_fn, message_fns, mod_items);
    let mut mutates: HashMap<String, bool> = HashMap::new();
    loop {
        let mut changed = false;
        for (name, block) in &all_fns {
            let already = *mutates.get(name).unwrap_or(&false);
            let cur = already
                || block_contains_mutating_call_with(block, &|n| *mutates.get(n).unwrap_or(&false));
            if cur && !already {
                mutates.insert(name.clone(), true);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    mutates
}

fn block_contains_mutating_call(block: &Block) -> bool {
    block_contains_mutating_call_with(block, &|_| false)
}

/// 判断合约函数是否包含修改状态的调用；若无则应在 ABI 中标记 mutates: false。
/// 注意：若需考虑“被调用的函数内部”也修改状态，应在 emit_abi 中使用 compute_mutates_fixpoint 的结果。
pub fn fn_mutates_state(f: &ItemFn) -> bool {
    block_contains_mutating_call(&f.block)
}

/// 从 mod 内 Item 或单个 struct 定义中提取 (struct_name, fields)。
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

/// 从 mod 内 Item 或单个 enum 定义中提取 (enum_name, 变体名列表)，仅当所有变体无字段时返回。
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

/// 从 enum 定义中提取 (enum_name, variants_with_fields)，每个变体为 (变体名, 字段类型列表)。
/// 用于生成 ABI 中带 fields 的 variant（如 AssetInfo: Native(Bytes), ERC20(Bytes, H256)）。
/// 若 enum 带泛型（如 EditType<T>），不在此处理，由 generic_enum_from_item 收集。
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
            enum_defs_with_fields.insert(name, variants);
        }
        if let Some((name, generic_def)) = generic_enum_from_item(item) {
            generic_enum_defs.insert(name, generic_def);
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
                                    enum_defs_with_fields.insert(name, variants);
                                }
                                if let Some((name, generic_def)) = generic_enum_from_item(&item) {
                                    generic_enum_defs.insert(name, generic_def);
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
                                                        enum_defs_with_fields.insert(name, variants);
                                                    }
                                                    if let Some((name, generic_def)) = generic_enum_from_item(&item) {
                                                        generic_enum_defs.insert(name, generic_def);
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
    message_fns: &[(ItemFn, [u8; 4])],
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

    let mut constructor_args = Vec::new();
    for arg in &constructor_fn.sig.inputs {
        let FnArg::Typed(pt) = arg else { continue };
        let arg_name = match pt.pat.as_ref() {
            Pat::Ident(pi) => pi.ident.to_string(),
            _ => continue,
        };
        if let Some((type_id, display_name)) = reg.ensure_type(pt.ty.as_ref()) {
            constructor_args.push(serde_json::json!({
                "label": arg_name,
                "type": { "displayName": display_name, "type": type_id }
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
        "docs": [],
        "label": constructor_fn.sig.ident.to_string(),
        "payable": false,
        "returnType": constructor_return,
        "selector": "0x00000000"
    })];

    let mutates_map = compute_mutates_fixpoint(constructor_fn, message_fns, mod_items);
    let mut messages = Vec::new();
    for (f, sel) in message_fns {
        let label = f.sig.ident.to_string();
        // 未检测到写操作时标为 false，避免仅含 .get/.desc_list 等只读调用的 message 被误判为 mutates true
        let mutates = mutates_map.get(&label).copied().unwrap_or(false);
        let selector = selector_hex(*sel);
        let mut args = Vec::new();
        for arg in &f.sig.inputs {
            let FnArg::Typed(pt) = arg else { continue };
            let arg_name = match pt.pat.as_ref() {
                Pat::Ident(pi) => pi.ident.to_string(),
                _ => continue,
            };
            if let Some((type_id, display_name)) = reg.ensure_type(pt.ty.as_ref()) {
                args.push(serde_json::json!({
                    "label": arg_name,
                    "type": { "displayName": display_name, "type": type_id }
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
            "docs": [],
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

    let id_account = syn::parse_str::<syn::Type>("Address")
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
        "accountId": { "displayName": ["AccountId"], "type": id_account },
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
