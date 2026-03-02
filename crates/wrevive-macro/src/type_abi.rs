//! 类型到 ABI 名称的映射（interface 扁平格式用）。

use crate::attrs;
use syn::Type;

/// Maps a Rust type to an ABI type name and optional length for abi.json interface.
pub fn type_to_abi(ty: &Type) -> Option<(String, Option<u32>)> {
    if let Type::Reference(r) = ty {
        return type_to_abi(&r.elem);
    }
    if let Type::Path(p) = ty {
        if let Some(id) = p.path.get_ident() {
            let name = id.to_string();
            match name.as_str() {
                "u8" | "u16" | "u32" | "u64" | "u128"
                | "i8" | "i16" | "i32" | "i64" | "i128"
                | "bool" => return Some((name, None)),
                "Address" => return Some(("Address".into(), Some(20))),
                _ => {}
            }
        }
        return Some((attrs::path_to_display_name(&p.path), None));
    }
    if let Type::Array(arr) = ty {
        if let Type::Path(inner) = *arr.elem.clone() {
            if inner.path.is_ident("u8") {
                if let syn::Expr::Lit(lit) = &arr.len {
                    if let syn::Lit::Int(n) = &lit.lit {
                        if let Ok(len) = n.base10_parse::<u32>() {
                            return Some(("AccountId".into(), Some(len)));
                        }
                    }
                }
            }
        }
    }
    // Option<T>, Vec<T>, 其他泛型等：统一标为 ScaleBytes
    Some(("ScaleBytes".into(), None))
}
