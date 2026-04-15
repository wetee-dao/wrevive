//! 动态类型注册表：按合约中出现的类型分配 id，生成 ink! ABI types 数组。

use crate::attrs;
use std::collections::HashMap;
use syn::{GenericArgument, Lit, PathArguments, Type};

/// 类型注册表：key -> id，types 按 id 顺序存放，每个合约独立编号 0,1,2,...
pub struct TypeReg {
    key_to_id: HashMap<String, u32>,
    types: Vec<serde_json::Value>,
    /// 已知 struct 名称 -> (字段名, 字段类型)，用于生成 composite 的 fields
    struct_defs: HashMap<String, Vec<(String, Type)>>,
    /// 已知 enum 名称 -> 变体名列表（仅无字段枚举），用于生成 variant 的 variants
    enum_defs: HashMap<String, Vec<String>>,
    /// 类型别名名称 -> 底层类型（如 NodeID -> u64），用于在 ABI 中生成带 path 的条目
    type_aliases: HashMap<String, Type>,
    /// 带字段的 enum：名称 -> [(变体名, 字段类型列表)]，用于生成 ABI 中带 fields 的 variant（如 AssetInfo）
    enum_defs_with_fields: HashMap<String, Vec<(String, Vec<Type>)>>,
    /// 带泛型的 enum：名称 -> (泛型参数名列表, [(变体名, 字段类型列表)])，实例化时推断类型生成 ABI（如 EditType<T>）
    generic_enum_defs: HashMap<String, (Vec<String>, Vec<(String, Vec<Type>)>)>,
}

fn strip_ref(ty: &Type) -> &Type {
    if let Type::Reference(r) = ty {
        return strip_ref(&r.elem);
    }
    ty
}

/// 判断类型是否为单段路径且标识与 param 相同（泛型参数占位，如 T）。
fn type_is_param(ty: &Type, param: &str) -> bool {
    let ty = strip_ref(ty);
    if let Type::Path(p) = ty {
        if p.path.segments.len() == 1 {
            let seg = p.path.segments.first().unwrap();
            if seg.ident == param {
                if let PathArguments::None = &seg.arguments {
                    return true;
                }
            }
        }
    }
    false
}

/// 将泛型参数占位类型替换为具体类型（用于 EditType<T> 实例化为 EditType<u64>）。
fn substitute_type(ty: &Type, param_names: &[String], type_args: &[Type]) -> Type {
    for (i, p) in param_names.iter().enumerate() {
        if let Some(concrete) = type_args.get(i) {
            if type_is_param(ty, p) {
                return concrete.clone();
            }
        }
    }
    ty.clone()
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
            // 带类型实参的单段路径（如 EditType<u64>）：key 含实参以区分不同实例化。
            if p.path.segments.len() == 1 {
                if let PathArguments::AngleBracketed(a) = &seg.arguments {
                    let args: Vec<String> = a
                        .args
                        .iter()
                        .filter_map(|arg| match arg {
                            GenericArgument::Type(t) => type_key(t),
                            _ => None,
                        })
                        .collect();
                    if !args.is_empty() {
                        return Some(format!("{}<{}>", seg.ident, args.join(",")));
                    }
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
                                // Do not special-case "AccountId" for byte arrays.
                                // Keep the display name as raw fixed-size bytes.
                                format!("[u8; {}]", len)
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
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            type_aliases: HashMap::new(),
            enum_defs_with_fields: HashMap::new(),
            generic_enum_defs: HashMap::new(),
        }
    }

    /// 注册从合约 mod 或 crate 源码收集到的 struct 定义（名称 -> 字段列表），用于生成 composite 的 fields。
    pub fn register_struct_defs(&mut self, defs: HashMap<String, Vec<(String, Type)>>) {
        for (name, fields) in defs {
            if !fields.is_empty() {
                self.struct_defs.insert(name, fields);
            }
        }
    }

    /// 注册从合约 mod 或 crate 源码收集到的 enum 定义（名称 -> 变体列表），用于生成 variant 的 variants。
    pub fn register_enum_defs(&mut self, defs: HashMap<String, Vec<String>>) {
        for (name, variants) in defs {
            if !variants.is_empty() {
                self.enum_defs.insert(name, variants);
            }
        }
    }

    /// 注册类型别名（如 NodeID = u64），ABI 中会生成 def=底层类型、path=[别名] 的条目。
    pub fn register_type_aliases(&mut self, aliases: HashMap<String, Type>) {
        for (name, ty) in aliases {
            self.type_aliases.insert(name, ty);
        }
    }

    /// 注册带字段的 enum（如 AssetInfo: Native(Bytes), ERC20(Bytes, H256)），ABI 中生成带 fields 的 variant。
    pub fn register_enum_defs_with_fields(&mut self, defs: HashMap<String, Vec<(String, Vec<Type>)>>) {
        for (name, variants) in defs {
            if !variants.is_empty() {
                self.enum_defs_with_fields.insert(name, variants);
            }
        }
    }

    /// 注册带泛型的 enum（如 EditType<T>），实例化（EditType<u64>）时用类型实参替换生成 ABI。
    pub fn register_generic_enum_defs(
        &mut self,
        defs: HashMap<String, (Vec<String>, Vec<(String, Vec<Type>)>)>,
    ) {
        for (name, val) in defs {
            if !val.1.is_empty() {
                self.generic_enum_defs.insert(name, val);
            }
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
        let (def, path_opt, params_opt) = self.build_def(ty)?;
        let id = self.next_id();
        let display = display_name_for_spec(ty);
        let mut type_obj = serde_json::json!({ "def": def });
        if let Some(path) = path_opt {
            type_obj["path"] = serde_json::json!(path);
        }
        if let Some(params) = params_opt {
            type_obj["params"] = params;
        }
        // 为 Option / Result 附加泛型参数信息，兼容 ink! ABI（例如 cloud.json 中的 params 字段）。
        if let Type::Path(p) = ty {
            if let Some(seg) = p.path.segments.last() {
                let name = seg.ident.to_string();
                if name == "Option" {
                    if let PathArguments::AngleBracketed(a) = &seg.arguments {
                        let inner = a.args.iter().find_map(|a| match a {
                            GenericArgument::Type(t) => Some(t),
                            _ => None,
                        });
                        if let Some(inner_ty) = inner {
                            if let Some((id_t, _)) = self.ensure_type(inner_ty) {
                                type_obj["params"] = serde_json::json!([
                                    { "name": "T", "type": id_t }
                                ]);
                            }
                        }
                    }
                } else if name == "Result" {
                    if let PathArguments::AngleBracketed(a) = &seg.arguments {
                        let mut args_it = a.args.iter().filter_map(|a| match a {
                            GenericArgument::Type(t) => Some(t),
                            _ => None,
                        });
                        if let (Some(t_ty), Some(e_ty)) = (args_it.next(), args_it.next()) {
                            if let Some((id_t, _)) = self.ensure_type(t_ty) {
                                if let Some((id_e, _)) = self.ensure_type(e_ty) {
                                    type_obj["params"] = serde_json::json!([
                                        { "name": "T", "type": id_t },
                                        { "name": "E", "type": id_e }
                                    ]);
                                }
                            }
                        }
                    }
                }
            }
        }
        self.types.push(serde_json::json!({ "id": id, "type": type_obj }));
        self.key_to_id.insert(key, id);
        Some((id, display))
    }

    fn build_def(&mut self, ty: &Type) -> Option<(serde_json::Value, Option<Vec<String>>, Option<serde_json::Value>)> {
        let ty = strip_ref(ty);
        if let Type::Path(p) = ty {
            if let Some(id) = p.path.get_ident() {
                let name = id.to_string();
                match name.as_str() {
                    "u8" | "u16" | "u32" | "u64" | "u128"
                    | "i8" | "i16" | "i32" | "i64" | "i128"
                    | "bool" => return Some((serde_json::json!({ "primitive": name }), None, None)),
                    "Address" => {
                        let id_arr = self.ensure_array_u8(20);
                        return Some((
                            serde_json::json!({ "composite": { "fields": [{ "type": id_arr, "typeName": "[u8; 20]" }] } }),
                            Some(vec!["primitive_types".into(), "H160".into()]),
                            None,
                        ));
                    }
                    // AccountId is treated as raw 32-byte fixed array (no special frontend handling),
                    // but we keep ink!-like ABI shape: a composite wrapper over `[u8; 32]` with path
                    // ["ink_primitives","types","AccountId"] (see examples/abi/*.json).
                    "AccountId" => {
                        let id_arr = self.ensure_array_u8(32);
                        return Some((
                            serde_json::json!({ "composite": { "fields": [{ "type": id_arr, "typeName": "[u8; 32]" }] } }),
                            Some(vec!["ink_primitives".into(), "types".into(), "AccountId".into()]),
                            None,
                        ));
                    }
                    "H256" => {
                        let id_arr = self.ensure_array_u8(32);
                        return Some((
                            serde_json::json!({ "composite": { "fields": [{ "type": id_arr, "typeName": "[u8; 32]" }] } }),
                            Some(vec!["primitive_types".into(), "H256".into()]),
                            None,
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
                        return Some((serde_json::json!({ "sequence": { "type": id_inner } }), None, None));
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
                                        { "index": 0, "name": "None" },
                                        { "name": "Some", "fields": [{ "type": id_ok }], "index": 1 }
                                    ]
                                }
                            }),
                            Some(vec!["Option".into()]),
                            None,
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
                                        { "name": "Ok", "fields": [{ "type": id_ok }], "index": 0 },
                                        { "name": "Err", "fields": [{ "type": id_err }], "index": 1 }
                                    ]
                                }
                            }),
                            Some(vec!["Result".into()]),
                            None,
                        ));
                    }
                }
                // 带泛型的 enum 实例化（如 EditType<u64>）：用类型实参替换后生成 variant，与 cloud.json 一致。
                if let Some((param_names, variants)) = self.generic_enum_defs.get(&name).cloned() {
                    if let PathArguments::AngleBracketed(a) = &seg.arguments {
                        let type_args: Vec<Type> = a
                            .args
                            .iter()
                            .filter_map(|arg| match arg {
                                GenericArgument::Type(t) => Some(t.clone()),
                                _ => None,
                            })
                            .collect();
                        if type_args.len() == param_names.len() {
                            let mut param_type_ids = Vec::new();
                            for t in &type_args {
                                if let Some((id, _)) = self.ensure_type(t) {
                                    param_type_ids.push(id);
                                }
                            }
                            if param_type_ids.len() != param_names.len() {
                                return None;
                            }
                            let mut vs = Vec::new();
                            for (idx, (vname, field_tys)) in variants.iter().enumerate() {
                                let mut field_entries = Vec::new();
                                for ft in field_tys {
                                    let concrete = substitute_type(ft, &param_names, &type_args);
                                    if let Some((type_id, _)) = self.ensure_type(&concrete) {
                                        let type_name =
                                            type_key(&concrete).unwrap_or_else(|| "ScaleBytes".into());
                                        field_entries.push(serde_json::json!({
                                            "type": type_id,
                                            "typeName": type_name
                                        }));
                                    }
                                }
                                vs.push(serde_json::json!({
                                    "name": vname,
                                    "index": idx as u8,
                                    "fields": field_entries
                                }));
                            }
                            let params_json: Vec<_> = param_names
                                .iter()
                                .zip(param_type_ids.iter())
                                .map(|(n, &id)| serde_json::json!({ "name": n, "type": id }))
                                .collect();
                            return Some((
                                serde_json::json!({ "variant": { "variants": vs } }),
                                Some(vec![name.clone()]),
                                Some(serde_json::json!(params_json)),
                            ));
                        }
                    }
                }
            }
            let name = attrs::path_to_display_name(&p.path);
            if name == "LangError" {
                return Some((
                    serde_json::json!({ "variant": { "variants": [] } }),
                    Some(vec!["ink".into(), "LangError".into()]),
                    None,
                ));
            }
            // 类型别名（如 NodeID = u64）：用底层类型的 def，path 为别名名，使 ABI 中生成该类型条目。
            if let Some(underlying) = self.type_aliases.get(&name).cloned() {
                if let Some((def, _, _)) = self.build_def(&underlying) {
                    return Some((def, Some(vec![name]), None));
                }
            }
            // 优先使用已收集到的 struct 定义生成 composite fields。
            if let Some(fields_def) = self.struct_defs.get(&name).cloned() {
                let mut field_entries = Vec::new();
                for (field_name, field_ty) in fields_def {
                    if let Some((type_id, _)) = self.ensure_type(&field_ty) {
                        let type_name = type_key(&field_ty).unwrap_or_else(|| "ScaleBytes".into());
                        field_entries.push(serde_json::json!({
                            "name": field_name,
                            "type": type_id,
                            "typeName": type_name
                        }));
                    }
                }
                if !field_entries.is_empty() {
                    return Some((
                        serde_json::json!({ "composite": { "fields": field_entries } }),
                        Some(vec![name]),
                        None,
                    ));
                }
            }
            // 带字段的 enum（如 AssetInfo: Native(Bytes), ERC20(Bytes, H256)）：生成带 fields 的 variant，与 subnet.json 一致。
            if let Some(variants_def) = self.enum_defs_with_fields.get(&name).cloned() {
                let mut vs = Vec::new();
                for (idx, (vname, field_tys)) in variants_def.into_iter().enumerate() {
                    let mut field_entries = Vec::new();
                    for ft in &field_tys {
                        if let Some((type_id, _)) = self.ensure_type(ft) {
                            let type_name = type_key(ft).unwrap_or_else(|| "ScaleBytes".into());
                            field_entries.push(serde_json::json!({
                                "type": type_id,
                                "typeName": type_name
                            }));
                        }
                    }
                    vs.push(serde_json::json!({
                        "name": vname,
                        "index": idx as u8,
                        "fields": field_entries
                    }));
                }
                return Some((
                    serde_json::json!({ "variant": { "variants": vs } }),
                    Some(vec![name]),
                    None,
                ));
            }
            // 无字段 enum：仅变体名列表。
            if let Some(variants_def) = self.enum_defs.get(&name).cloned() {
                let mut vs = Vec::new();
                for (idx, vname) in variants_def.into_iter().enumerate() {
                    vs.push(serde_json::json!({
                        "fields": [],
                        "index": idx as u8,
                        "name": vname,
                    }));
                }
                return Some((
                    serde_json::json!({ "variant": { "variants": vs } }),
                    Some(vec![name]),
                    None,
                ));
            }
            // 默认退回到空 composite，占位但不带字段/变体。
            return Some((
                serde_json::json!({ "composite": {} }),
                Some(vec![name]),
                None,
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
                return Some((serde_json::json!({ "tuple": [] }), None, None));
            }
            let mut elem_ids = Vec::new();
            for e in &t.elems {
                let (id, _) = self.ensure_type(e)?;
                elem_ids.push(id);
            }
            return Some((
                serde_json::json!({ "tuple": elem_ids }),
                None,
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

    /// LangError 与 pod.json 一致：path ["ink_primitives", "LangError"]，至少一个变体。
    pub fn ensure_lang_error(&mut self) -> u32 {
        let key = "LangError".to_string();
        if let Some(&id) = self.key_to_id.get(&key) {
            return id;
        }
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": {
                "def": {
                    "variant": {
                        "variants": [
                            { "index": 1, "name": "CouldNotReadInput" }
                        ]
                    }
                },
                "path": ["ink_primitives", "LangError"]
            }
        }));
        self.key_to_id.insert(key, id);
        id
    }

    /// 构造函数返回值类型：Result<(), LangError>，与 pod.json 一致（path ["Result"] + params T,E）。
    pub fn ensure_constructor_result(&mut self) -> (u32, Vec<String>) {
        let unit_id = self.ensure_unit();
        let lang_id = self.ensure_lang_error();
        let key = "ConstructorResult".to_string();
        if let Some(&id) = self.key_to_id.get(&key) {
            return (id, vec!["Result".into()]);
        }
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": {
                "def": {
                    "variant": {
                        "variants": [
                            { "name": "Ok", "fields": [{ "type": unit_id }], "index": 0 },
                            { "name": "Err", "fields": [{ "type": lang_id }], "index": 1 }
                        ]
                    }
                },
                "params": [
                    { "name": "T", "type": unit_id },
                    { "name": "E", "type": lang_id }
                ],
                "path": ["Result"]
            }
        }));
        self.key_to_id.insert(key, id);
        (id, vec!["Result".into()])
    }

    /// Message 返回 Result<(), LangError>，与 pod.json 一致（path ["Result"] + params）。
    pub fn ensure_message_result_unit(&mut self) -> (u32, Vec<String>) {
        let key = "Result<(),LangError>".to_string();
        if let Some(&id) = self.key_to_id.get(&key) {
            return (id, vec!["Result".into()]);
        }
        let unit_id = self.ensure_unit();
        let lang_id = self.ensure_lang_error();
        let id = self.next_id();
        self.types.push(serde_json::json!({
            "id": id,
            "type": {
                "def": {
                    "variant": {
                        "variants": [
                            { "name": "Ok", "fields": [{ "type": unit_id }], "index": 0 },
                            { "name": "Err", "fields": [{ "type": lang_id }], "index": 1 }
                        ]
                    }
                },
                "params": [
                    { "name": "T", "type": unit_id },
                    { "name": "E", "type": lang_id }
                ],
                "path": ["Result"]
            }
        }));
        self.key_to_id.insert(key, id);
        (id, vec!["Result".into()])
    }

    /// 为任意类型 T 注册 Result<T, LangError>，与 pod.json 一致（path ["Result"] + params T,E）。
    pub fn ensure_message_result(&mut self, ty: &Type) -> Option<(u32, Vec<String>)> {
        let key = format!("Result<{},LangError>", type_key(ty)?);
        if let Some(&id) = self.key_to_id.get(&key) {
            return Some((id, vec!["Result".into()]));
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
                            { "name": "Ok", "fields": [{ "type": id_ok }], "index": 0 },
                            { "name": "Err", "fields": [{ "type": lang_id }], "index": 1 }
                        ]
                    }
                },
                "params": [
                    { "name": "T", "type": id_ok },
                    { "name": "E", "type": lang_id }
                ],
                "path": ["Result"]
            }
        }));
        self.key_to_id.insert(key, id);
        Some((id, vec!["Result".into()]))
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
