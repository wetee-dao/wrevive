//! ink! 风格 ABI JSON 生成与写入 target/contract/{name}.json。

use crate::type_reg::TypeReg;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{
    Block, Expr, FnArg, ItemFn, Pat, ReturnType, Stmt, Type,
};

/// 合约名中 `-` 改为 `_`，避免 go-ink-gen 生成非法 Go 标识符（expected ';', found '-'）。
fn contract_name_go_safe(name: &str) -> String {
    name.replace('-', "_")
}

/// 是否为 Result<T,E> 类型（go-ink-gen 要求 returnType 为 Variant，非 Result 的需包装成 MessageResult）。
fn is_result_type(ty: &Type) -> bool {
    let Type::Path(p) = ty else { return false };
    let Some(seg) = p.path.segments.last() else { return false };
    seg.ident == "Result"
}

/// 将 4 字节 selector 格式化为十六进制字符串，用于 ABI 的 selector 字段。
pub fn selector_hex(sel: [u8; 4]) -> String {
    format!("0x{:02x}{:02x}{:02x}{:02x}", sel[0], sel[1], sel[2], sel[3])
}

/// 视为修改状态的方法名：Storage/Mapping::set、List/List2D::insert、deposit_event 等。
fn is_mutating_method(name: &str) -> bool {
    matches!(
        name,
        "set" | "insert" | "remove" | "clear" | "deposit_event"
    )
}

/// 递归检查表达式中是否包含修改状态的调用（.set / .insert / .remove / .clear / .deposit_event）。
fn expr_contains_mutating_call(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            let name = m.method.to_string();
            if is_mutating_method(&name) {
                return true;
            }
            if expr_contains_mutating_call(&m.receiver) {
                return true;
            }
            for arg in &m.args {
                if expr_contains_mutating_call(arg) {
                    return true;
                }
            }
            false
        }
        Expr::Block(b) => block_contains_mutating_call(&b.block),
        Expr::If(e) => {
            expr_contains_mutating_call(&e.cond)
                || block_contains_mutating_call(&e.then_branch)
                || e.else_branch
                    .as_ref()
                    .map(|(_, e)| expr_contains_mutating_call(e))
                    .unwrap_or(false)
        }
        Expr::Match(e) => {
            if expr_contains_mutating_call(&e.expr) {
                return true;
            }
            for arm in &e.arms {
                if expr_contains_mutating_call(&arm.body) {
                    return true;
                }
            }
            false
        }
        Expr::Call(c) => {
            if expr_contains_mutating_call(&c.func) {
                return true;
            }
            for arg in &c.args {
                if expr_contains_mutating_call(arg) {
                    return true;
                }
            }
            false
        }
        Expr::Let(e) => expr_contains_mutating_call(&e.expr),
        Expr::Return(e) => e
            .expr
            .as_ref()
            .map_or(false, |e| expr_contains_mutating_call(e)),
        Expr::Assign(a) => {
            expr_contains_mutating_call(&a.left) || expr_contains_mutating_call(&a.right)
        }
        Expr::Closure(c) => expr_contains_mutating_call(&c.body),
        Expr::Loop(l) => block_contains_mutating_call(&l.body),
        Expr::While(w) => {
            expr_contains_mutating_call(&w.cond) || block_contains_mutating_call(&w.body)
        }
        Expr::ForLoop(f) => {
            expr_contains_mutating_call(&f.expr) || block_contains_mutating_call(&f.body)
        }
        Expr::Unsafe(u) => block_contains_mutating_call(&u.block),
        Expr::Async(a) => block_contains_mutating_call(&a.block),
        Expr::TryBlock(t) => block_contains_mutating_call(&t.block),
        _ => false,
    }
}

fn block_contains_mutating_call(block: &Block) -> bool {
    for stmt in &block.stmts {
        if stmt_contains_mutating_call(stmt) {
            return true;
        }
    }
    false
}

fn stmt_contains_mutating_call(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(expr, _) => expr_contains_mutating_call(expr),
        Stmt::Local(local) => local
            .init
            .as_ref()
            .map(|init| expr_contains_mutating_call(&init.expr))
            .unwrap_or(false),
        Stmt::Item(_) | Stmt::Macro(_) => false,
    }
}

/// 判断合约函数是否包含修改状态的调用；若无则应在 ABI 中标记 mutates: false。
pub fn fn_mutates_state(f: &ItemFn) -> bool {
    block_contains_mutating_call(&f.block)
}

/// 根据合约名、构造函数与 message 列表生成 ink! 风格 ABI JSON。
/// 输出路径：以当前项目根或 workspace 根为基准，写入 `target/contract/{name}.json`。
/// 返回 `Ok(())` 表示成功，`Err(msg)` 表示失败（调用方负责打印结果）。
pub fn emit_abi(
    contract_name: &str,
    constructor_fn: &ItemFn,
    message_fns: &[(ItemFn, [u8; 4])],
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
    let base_dir = find_workspace_root(&manifest_abs).unwrap_or(manifest_abs);
    let out_dir = base_dir.join("target");
    fs::create_dir_all(&out_dir).map_err(|e| {
        format!("创建目录 {} 失败: {}", out_dir.display(), e)
    })?;
    let out_path = out_dir.join(format!("{}.json", contract_name));

    let mut reg = TypeReg::new();

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

    let mut messages = Vec::new();
    for (f, sel) in message_fns {
        let label = f.sig.ident.to_string();
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
            ReturnType::Type(_, ty) => {
                let (type_id, display_name) = if is_result_type(ty) {
                    reg.ensure_type(ty).unwrap_or((0, vec!["ScaleBytes".into()]))
                } else {
                    reg.ensure_message_result(ty).unwrap_or_else(|| {
                        reg.ensure_type(ty).unwrap_or((0, vec!["ScaleBytes".into()]))
                    })
                };
                serde_json::json!({ "displayName": display_name, "type": type_id })
            }
        };
        let mutates = fn_mutates_state(f);
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
