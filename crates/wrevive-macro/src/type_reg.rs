//! 动态类型注册表：按合约中出现的类型分配 id，生成 ink! ABI types 数组。

use crate::attrs;
use std::collections::HashMap;
use syn::{GenericArgument, Lit, PathArguments, Type};

/// 类型注册表：key -> id，types 按 id 顺序存放，每个合约独立编号 0,1,2,...
pub struct TypeReg {
    key_to_id: HashMap<String, u32>,
    types: Vec<serde_json::Value>,
}

fn strip_ref(ty: &Type) -> &Type {
    if let Type::Reference(r) = ty {
        return strip_ref(&r.elem);
    }
    ty
}

fn type_key(ty: &Type) -> Option<String> {
    let ty = strip_ref(ty);
    if let Type::Path(p) = ty {
        if let Some(id) = p.path.get_ident() {
            let name = id.to_string();
            match name.as_str() {
                "u8" | "u16" | "u32" | "u64" | "u128"
                | "i8" | "i16" | "i32" | "i64" | "i128"
                | "bool" => return Some(name),
                "Address" => return Some("Address".into()),
                _ => {}
            }
        }
        if let Some(seg) = p.path.segments.last() {
            let name = seg.ident.to_string();
            if name == "Vec" {
                if let PathArguments::AngleBracketed(a) = &seg.arguments {
                    let first = a.args.iter().find_map(|a| match a {
                        GenericArgument::Type(t) => Some(t),
                        _ => None,
                    })?;
                    return Some(format!("Vec<{}>", type_key(first)?));
                }
            }
            if name == "Option" {
                if let PathArguments::AngleBracketed(a) = &seg.arguments {
                    let first = a.args.iter().find_map(|a| match a {
                        GenericArgument::Type(t) => Some(t),
                        _ => None,
                    })?;
                    return Some(format!("Option<{}>", type_key(first)?));
                }
            }
            if name == "Result" {
                if let PathArguments::AngleBracketed(a) = &seg.arguments {
                    let mut args = a.args.iter().filter_map(|a| match a {
                        GenericArgument::Type(t) => Some(t),
                        _ => None,
                    });
                    let t = args.next()?;
                    let e = args.next()?;
                    return Some(format!("Result<{},{}>", type_key(t)?, type_key(e)?));
                }
            }
        }
        return Some(attrs::path_to_display_name(&p.path));
    }
    if let Type::Array(arr) = ty {
        if let Type::Path(inner) = *arr.elem.clone() {
            if inner.path.is_ident("u8") {
                if let syn::Expr::Lit(lit) = &arr.len {
                    if let Lit::Int(n) = &lit.lit {
                        if let Ok(len) = n.base10_parse::<u32>() {
                            return Some(format!("[u8;{}]", len));
                        }
                    }
                }
            }
        }
    }
    if let Type::Tuple(t) = ty {
        if t.elems.is_empty() {
            return Some("()".into());
        }
        let parts: Vec<String> = t.elems.iter().filter_map(|e| type_key(e)).collect();
        if parts.len() == t.elems.len() {
            return Some(format!("({})", parts.join(",")));
        }
    }
    None
}

fn display_name_for_spec(ty: &Type) -> Vec<String> {
    let ty = strip_ref(ty);
    if let Type::Tuple(t) = ty {
        if t.elems.is_empty() {
            return vec!["()".into()];
        }
    }
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            let name = seg.ident.to_string();
            if name == "Result" {
                return vec!["ink".into(), "MessageResult".into()];
            }
            if name == "Option" {
                return vec!["Option".into()];
            }
            if name == "Vec" {
                return vec!["Vec".into()];
            }
        }
        return vec![attrs::path_to_display_name(&p.path)];
    }
    if let Type::Array(arr) = ty {
        if let Type::Path(inner) = *arr.elem.clone() {
            if inner.path.is_ident("u8") {
                if let syn::Expr::Lit(lit) = &arr.len {
                    if let Lit::Int(n) = &lit.lit {
                        if let Ok(len) = n.base10_parse::<u32>() {
                            return vec![if len == 20 {
                                "Address".into()
                            } else if len == 32 {
                                "H256".into()
                            } else {
                                "AccountId".into()
                            }];
                        }
                    }
                }
            }
        }
    }
    vec!["ScaleBytes".into()]
}

impl TypeReg {
    pub fn new() -> Self {
        Self {
            key_to_id: HashMap::new(),
            types: Vec::new(),
        }
    }

    fn next_id(&self) -> u32 {
        self.types.len() as u32
    }

    /// 确保类型已注册，返回 (type_id, display_name)。
    pub fn ensure_type(&mut self, ty: &Type) -> Option<(u32, Vec<String>)> {
        let ty = strip_ref(ty);
        let key = type_key(ty)?;
        if let Some(&id) = self.key_to_id.get(&key) {
            let display = display_name_for_spec(ty);
            return Some((id, display));
        }
        let (def, path_opt) = self.build_def(ty)?;
        let id = self.next_id();
        let display = display_name_for_spec(ty);
        let mut type_obj = serde_json::json!({ "def": def });
        if let Some(path) = path_opt {
            type_obj["path"] = serde_json::json!(path);
        }
        self.types.push(serde_json::json!({ "id": id, "type": type_obj }));
        self.key_to_id.insert(key, id);
        Some((id, display))
    }

    fn build_def(&mut self, ty: &Type) -> Option<(serde_json::Value, Option<Vec<String>>)> {
        let ty = strip_ref(ty);
        if let Type::Path(p) = ty {
            if let Some(id) = p.path.get_ident() {
                let name = id.to_string();
                match name.as_str() {
                    "u8" | "u16" | "u32" | "u64" | "u128"
                    | "i8" | "i16" | "i32" | "i64" | "i128"
                    | "bool" => return Some((serde_json::json!({ "primitive": name }), None)),
                    "Address" => {
                        let id_arr = self.ensure_array_u8(20);
                        return Some((
                            serde_json::json!({ "composite": { "fields": [{ "type": id_arr, "typeName": "[u8; 20]" }] } }),
                            Some(vec!["primitive_types".into(), "H160".into()]),
                        ));
                    }
                    "H256" => {
                        let id_arr = self.ensure_array_u8(32);
                        return Some((
                            serde_json::json!({ "composite": { "fields": [{ "type": id_arr, "typeName": "[u8; 32]" }] } }),
                            Some(vec!["primitive_types".into(), "H256".into()]),
                        ));
                    }
                    _ => {}
                }
            }
            if let Some(seg) = p.path.segments.last() {
                let name = seg.ident.to_string();
                if name == "Vec" {
                    if let PathArguments::AngleBracketed(a) = &seg.arguments {
                        let first = a.args.iter().find_map(|a| match a {
                            GenericArgument::Type(t) => Some(t),
                            _ => None,
                        })?;
                        let (id_inner, _) = self.ensure_type(first)?;
                        return Some((serde_json::json!({ "sequence": { "type": id_inner } }), None));
                    }
                }
                if name == "Option" {
                    if let PathArguments::AngleBracketed(a) = &seg.arguments {
                        let first = a.args.iter().find_map(|a| match a {
                            GenericArgument::Type(t) => Some(t),
                            _ => None,
                        })?;
                        let (id_ok, _) = self.ensure_type(first)?;
                        return Some((
                            serde_json::json!({
                                "variant": {
                                    "variants": [
                                        { "name": "None", "fields": [], "index": 0 },
                                        { "name": "Some", "fields": [{ "type": id_ok, "typeName": "T" }], "index": 1 }
                                    ]
                                }
                            }),
                            Some(vec!["Option".into()]),
                        ));
                    }
                }
                if name == "Result" {
                    if let PathArguments::AngleBracketed(a) = &seg.arguments {
                        let mut args = a.args.iter().filter_map(|a| match a {
                            GenericArgument::Type(t) => Some(t),
                            _ => None,
                        });
                        let t = args.next()?;
                        let e = args.next()?;
                        let (id_ok, _) = self.ensure_type(t)?;
                        let (id_err, _) = self.ensure_type(e)?;
                        return Some((
                            serde_json::json!({
                                "variant": {
                                    "variants": [
                                        { "name": "Ok", "fields": [{ "type": id_ok, "typeName": "T" }], "index": 0 },
                                        { "name": "Err", "fields": [{ "type": id_err, "typeName": "E" }], "index": 1 }
                                    ]
                                }
                            }),
                            Some(vec!["ink".into(), "MessageResult".into()]),
                        ));
                    }
                }
            }
            let name = attrs::path_to_display_name(&p.path);
            if name == "LangError" {
                return Some((
                    serde_json::json!({ "variant": { "variants": [] } }),
                    Some(vec!["ink".into(), "LangError".into()]),
                ));
            }
            return Some((
                serde_json::json!({ "composite": {} }),
                Some(vec![name]),
            ));
        }
        if let Type::Array(arr) = ty {
            if let Type::Path(inner) = *arr.elem.clone() {
                if inner.path.is_ident("u8") {
                    if let syn::Expr::Lit(lit) = &arr.len {
                        if let Lit::Int(n) = &lit.lit {
                            if let Ok(len) = n.base10_parse::<u32>() {
                                let id_u8 = self.ensure_u8();
                                return Some((
                                    serde_json::json!({ "array": { "len": len, "type": id_u8 } }),
                                    None,
                                ));
                            }
                        }
                    }
                }
            }
        }
        if let Type::Tuple(t) = ty {
            if t.elems.is_empty() {
                return Some((serde_json::json!({ "tuple": [] }), None));
            }
            let mut elem_ids = Vec::new();
            for e in &t.elems {
                let (id, _) = self.ensure_type(e)?;
                elem_ids.push(id);
            }
            return Some((
                serde_json::json!({ "tuple": elem_ids }),
                None,
            ));
        }
        None
    }

    fn ensure_u8(&mut self) -> u32 {
        let key = "u8".to_string();
        if let Some(&id) = self.key_to_id.get(&key) {
            return id;
        }
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": { "def": { "primitive": "u8" } }
        }));
        self.key_to_id.insert(key, id);
        id
    }

    fn ensure_array_u8(&mut self, len: u32) -> u32 {
        let key = format!("[u8;{}]", len);
        if let Some(&id) = self.key_to_id.get(&key) {
            return id;
        }
        let id_u8 = self.ensure_u8();
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": { "def": { "array": { "len": len, "type": id_u8 } } }
        }));
        self.key_to_id.insert(key, id);
        id
    }

    fn ensure_unit(&mut self) -> u32 {
        let key = "()".to_string();
        if let Some(&id) = self.key_to_id.get(&key) {
            return id;
        }
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": { "def": { "tuple": [] } }
        }));
        self.key_to_id.insert(key, id);
        id
    }

    pub fn ensure_lang_error(&mut self) -> u32 {
        let key = "LangError".to_string();
        if let Some(&id) = self.key_to_id.get(&key) {
            return id;
        }
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": {
                "def": { "variant": { "variants": [] } },
                "path": ["ink", "LangError"]
            }
        }));
        self.key_to_id.insert(key, id);
        id
    }

    pub fn ensure_constructor_result(&mut self) -> (u32, Vec<String>) {
        let _ = self.ensure_unit();
        let lang_id = self.ensure_lang_error();
        let key = "ConstructorResult".to_string();
        if let Some(&id) = self.key_to_id.get(&key) {
            return (id, vec!["ink_primitives".into(), "ConstructorResult".into()]);
        }
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": {
                "def": {
                    "variant": {
                        "variants": [
                            { "name": "Ok", "fields": [], "index": 0 },
                            { "name": "Err", "fields": [{ "type": lang_id, "typeName": "LangError" }], "index": 1 }
                        ]
                    }
                },
                "path": ["ink_primitives", "ConstructorResult"]
            }
        }));
        self.key_to_id.insert(key, id);
        (id, vec!["ink_primitives".into(), "ConstructorResult".into()])
    }

    pub fn ensure_message_result_unit(&mut self) -> (u32, Vec<String>) {
        let key = "Result<(),LangError>".to_string();
        if let Some(&id) = self.key_to_id.get(&key) {
            return (id, vec!["ink".into(), "MessageResult".into()]);
        }
        let _ = self.ensure_unit();
        let lang_id = self.ensure_lang_error();
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": {
                "def": {
                    "variant": {
                        "variants": [
                            { "name": "Ok", "fields": [], "index": 0 },
                            { "name": "Err", "fields": [{ "type": lang_id, "typeName": "LangError" }], "index": 1 }
                        ]
                    }
                },
                "path": ["ink", "MessageResult"]
            }
        }));
        self.key_to_id.insert(key, id);
        (id, vec!["ink".into(), "MessageResult".into()])
    }

    /// 为任意类型 T 注册 Result<T, LangError>，使 go-ink-gen 的 GetReturnValue 能解析（要求 returnType 为 Variant）。
    pub fn ensure_message_result(&mut self, ty: &Type) -> Option<(u32, Vec<String>)> {
        let key = format!("Result<{},LangError>", type_key(ty)?);
        if let Some(&id) = self.key_to_id.get(&key) {
            return Some((id, vec!["ink".into(), "MessageResult".into()]));
        }
        let (id_ok, _) = self.ensure_type(ty)?;
        let lang_id = self.ensure_lang_error();
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": {
                "def": {
                    "variant": {
                        "variants": [
                            { "name": "Ok", "fields": [{ "type": id_ok, "typeName": "T" }], "index": 0 },
                            { "name": "Err", "fields": [{ "type": lang_id, "typeName": "LangError" }], "index": 1 }
                        ]
                    }
                },
                "path": ["ink", "MessageResult"]
            }
        }));
        self.key_to_id.insert(key, id);
        Some((id, vec!["ink".into(), "MessageResult".into()]))
    }

    pub fn ensure_hash(&mut self) -> u32 {
        let key = "Hash".to_string();
        if let Some(&id) = self.key_to_id.get(&key) {
            return id;
        }
        let id_arr = self.ensure_array_u8(32);
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": {
                "def": { "composite": { "fields": [{ "type": id_arr, "typeName": "[u8; 32]" }] } },
                "path": ["ink_primitives", "types", "Hash"]
            }
        }));
        self.key_to_id.insert(key, id);
        id
    }

    pub fn types_array(&self) -> &[serde_json::Value] {
        &self.types
    }
}
