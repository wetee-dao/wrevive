//! Storage/Mapping/List/List2D 的 prefix 提取与去重（revive_contract 编译期检查用）。
//! 也提供 lit_to_prefix_bytes / blake2s_prefix_4_bytes 供 storage!/mapping!/list! 宏使用。

use blake2::Blake2s256;
use digest::Digest;
use syn::{Expr, Item, Lit};

/// 从字符串或字节串字面量得到字节（供 storage!/mapping! 哈希用）。
pub fn lit_to_prefix_bytes(lit: &Lit) -> Option<Vec<u8>> {
    match lit {
        Lit::Str(s) => Some(s.value().into_bytes()),
        Lit::ByteStr(bs) => Some(bs.value()),
        _ => None,
    }
}

/// 将 prefix 字节用 Blake2s256 哈希后取前 4 字节。
pub fn blake2s_prefix_4_bytes(bytes: &[u8]) -> [u8; 4] {
    let hash = Blake2s256::digest(bytes);
    [hash[0], hash[1], hash[2], hash[3]]
}

/// 若 expr 是 `X::new(b"prefix")` 或 `X::new(&[b0,b1,b2,b3])`，返回 Some(prefix 字节)。
pub fn extract_prefix_from_new_call(expr: &Expr) -> Option<Vec<u8>> {
    let call = match expr {
        Expr::Call(c) => c,
        _ => return None,
    };
    let path = match &*call.func {
        Expr::Path(p) => &p.path,
        _ => return None,
    };
    if path.segments.last()?.ident != "new" {
        return None;
    }
    let first_arg = call.args.first()?;
    let inner = match first_arg {
        Expr::Reference(r) => &*r.expr,
        other => other,
    };
    if let Expr::Lit(l) = inner {
        return match &l.lit {
            Lit::ByteStr(bs) => Some(bs.value()),
            Lit::Str(s) => Some(s.value().into_bytes()),
            _ => None,
        };
    }
    if let Expr::Array(arr) = inner {
        let mut bytes = Vec::with_capacity(arr.elems.len());
        for e in &arr.elems {
            if let Expr::Lit(l) = e {
                if let Lit::Byte(b) = &l.lit {
                    bytes.push(b.value());
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        return Some(bytes);
    }
    None
}

/// 若 expr 是未展开的 `storage!(...)` 或 `mapping!(...)`，返回 Blake2s 前 4 字节。
pub fn extract_prefix_from_storage_mapping_macro(expr: &Expr) -> Option<Vec<u8>> {
    let mac = match expr {
        Expr::Macro(m) => m,
        _ => return None,
    };
    let seg = mac.mac.path.segments.last()?;
    let name = seg.ident.to_string();
    if name != "storage" && name != "mapping" {
        return None;
    }
    let lit: Lit = syn::parse2(mac.mac.tokens.clone()).ok()?;
    let bytes = lit_to_prefix_bytes(&lit)?;
    Some(blake2s_prefix_4_bytes(&bytes).to_vec())
}

/// 若 expr 是未展开的 `list!(...)`，返回两个 prefix：_id、_items。
pub fn extract_prefixes_from_list_macro(expr: &Expr) -> Option<Vec<Vec<u8>>> {
    let mac = match expr {
        Expr::Macro(m) => m,
        _ => return None,
    };
    let seg = mac.mac.path.segments.last()?;
    if seg.ident != "list" {
        return None;
    }
    let lit: Lit = syn::parse2(mac.mac.tokens.clone()).ok()?;
    let bytes = lit_to_prefix_bytes(&lit)?;
    let p_id = blake2s_prefix_4_bytes(&bytes).to_vec();
    let p_items = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_items"].concat()).to_vec();
    Some(vec![p_id, p_items])
}

/// 若 expr 是未展开的 `list_2d!(...)`，返回四个 prefix。
pub fn extract_prefixes_from_list_2d_macro(expr: &Expr) -> Option<Vec<Vec<u8>>> {
    let mac = match expr {
        Expr::Macro(m) => m,
        _ => return None,
    };
    let seg = mac.mac.path.segments.last()?;
    if seg.ident != "list_2d" {
        return None;
    }
    let lit: Lit = syn::parse2(mac.mac.tokens.clone()).ok()?;
    let bytes = lit_to_prefix_bytes(&lit)?;
    let p_k1 = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_k1"].concat()).to_vec();
    let p_len = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_len"].concat()).to_vec();
    let p_k2 = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_k2"].concat()).to_vec();
    let p_store = blake2s_prefix_4_bytes(&[bytes.as_slice(), b"_store"].concat()).to_vec();
    Some(vec![p_k1, p_len, p_k2, p_store])
}

/// 从合约 mod 的 Item 中提取所有 Storage/Mapping/List/List2D 的 prefix，用于去重。
pub fn prefixes_from_item(item: &Item) -> Option<Vec<(Vec<u8>, String)>> {
    let s = match item {
        Item::Static(s) => s,
        _ => return None,
    };
    let name = s.ident.to_string();
    if let Some(p) = extract_prefix_from_new_call(&s.expr) {
        return Some(vec![(p, name)]);
    }
    if let Some(p) = extract_prefix_from_storage_mapping_macro(&s.expr) {
        return Some(vec![(p, name)]);
    }
    if let Some(ps) = extract_prefixes_from_list_macro(&s.expr) {
        return Some(ps.into_iter().map(|p| (p, name.clone())).collect());
    }
    if let Some(ps) = extract_prefixes_from_list_2d_macro(&s.expr) {
        return Some(ps.into_iter().map(|p| (p, name.clone())).collect());
    }
    None
}
