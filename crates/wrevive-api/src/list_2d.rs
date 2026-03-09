//! # List2D
//!
//! Two-dimensional list: grouped by K1, each group has an auto-increment Ix (u8/u16/u32/u64) index.
//! 二维列表：按 K1 分组，每组内为自增 Ix 索引的列表。
//! Layout: k1->id, k1_length, k2_next_id per id, store (id, k2)->value; see primitives define_double_map_base. Use list_2d! macro.

#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;

use crate::env::Env;
use crate::list::ListIndex;
use crate::mapping::Mapping;
use crate::Storage;

/// 二维列表：外层 key K1，内层自增索引 Ix（u8/u16/u32/u64）。K1 首次出现时分配一个 id，该 id 下 k2 从 0 自增。
#[derive(Clone, Copy)]
pub struct List2D<K1, Ix, V> {
    k1_to_id: Mapping<K1, Ix>,
    k1_length: Storage<Ix>,
    k2_next_id: Mapping<Ix, Ix>,
    store: Mapping<(Ix, Ix), V>,
}

impl<K1, Ix, V> List2D<K1, Ix, V>
where
    K1: parity_scale_codec::Encode + parity_scale_codec::Decode,
    Ix: ListIndex,
    V: parity_scale_codec::Encode + parity_scale_codec::Decode,
{
    /// 四个 prefix：k1_to_id, k1_length, k2_next_id, store。
    pub const fn new(
        prefix_k1_to_id: &'static [u8],
        prefix_k1_length: &'static [u8],
        prefix_k2_next_id: &'static [u8],
        prefix_store: &'static [u8],
    ) -> Self {
        Self {
            k1_to_id: Mapping::new(prefix_k1_to_id),
            k1_length: Storage::new(prefix_k1_length),
            k2_next_id: Mapping::new(prefix_k2_next_id),
            store: Mapping::new(prefix_store),
        }
    }

    /// 该 K1 下下一个将分配的 k2（即当前长度）。
    pub fn next_id(&self, api: &mut impl Env, k1: &K1) -> Ix {
        let id = match self.k1_to_id.get(api, k1) {
            Some(id) => id,
            None => return Ix::default(),
        };
        self.k2_next_id.get(api, &id).unwrap_or(Ix::default())
    }

    /// 该 K1 下的条目数量（与 next_id 一致）。
    pub fn len(&self, api: &mut impl Env, k1: &K1) -> Ix {
        self.next_id(api, k1)
    }

    /// Insert one entry under k1; returns the allocated k2.
    /// 在 k1 下插入一条记录，返回分配的 k2。
    pub fn insert(&self, api: &mut impl Env, k1: &K1, value: &V) -> Option<Ix> {
        let mut id = self.k1_to_id.get(api, k1);
        if id.is_none() {
            let len = self.k1_length.get(api).unwrap_or(Ix::default());
            id = Some(len);
            self.k1_to_id.set(api, k1, &len);
            let next_len = len.checked_next()?;
            self.k1_length.set(api, &next_len);
        }
        let id_val = id?;
        let next_id = self.k2_next_id.get(api, &id_val).unwrap_or(Ix::default());
        let new_next_id = next_id.checked_next()?;
        self.k2_next_id.set(api, &id_val, &new_next_id);
        let key = (id_val, next_id);
        self.store.set(api, &key, value);
        Some(next_id)
    }

    /// 更新 (k1, k2) 对应的值。
    pub fn update(&self, api: &mut impl Env, k1: &K1, k2: Ix, value: &V) -> Option<()> {
        let id = self.k1_to_id.get(api, k1)?;
        let key = (id, k2);
        self.store.set(api, &key, value)
    }

    /// 清除 (k1, k2) 对应的值（不改变该 k1 的 len/next_id）。
    pub fn clear(&self, api: &mut impl Env, k1: &K1, k2: Ix) -> Option<()> {
        let id = self.k1_to_id.get(api, k1)?;
        let key = (id, k2);
        self.store.clear(api, &key)
    }

    /// 按 (k1, k2) 取值。
    pub fn get(&self, api: &mut impl Env, k1: &K1, k2: Ix) -> Option<V> {
        let id = self.k1_to_id.get(api, k1)?;
        let key = (id, k2);
        self.store.get(api, &key)
    }

    /// 分页列表（升序）：k1 下从 start_key 起取最多 size 条。
    pub fn list(&self, api: &mut impl Env, k1: &K1, start_key: Ix, size: u32) -> Vec<(Ix, V)> {
        let id = match self.k1_to_id.get(api, k1) {
            Some(i) => i,
            None => return Vec::new(),
        };
        let total_len = self.k2_next_id.get(api, &id).unwrap_or(Ix::default());
        let mut out = Vec::new();
        if size == 0 {
            return out;
        }
        let mut k2 = start_key;
        for _ in 0..size {
            if k2 >= total_len {
                break;
            }
            let key = (id, k2);
            if let Some(v) = self.store.get(api, &key) {
                out.push((k2, v));
            }
            k2 = match k2.checked_next() {
                Some(n) => n,
                None => break,
            };
        }
        out
    }

    /// 分页列表（降序）：k1 下从 start_key_ 起向前取最多 size 条；None 表示从末尾开始。
    pub fn desc_list(
        &self,
        api: &mut impl Env,
        k1: &K1,
        start_key_: Option<Ix>,
        size: u32,
    ) -> Vec<(Ix, V)> {
        let id = match self.k1_to_id.get(api, k1) {
            Some(i) => i,
            None => return Vec::new(),
        };
        let total_len = self.k2_next_id.get(api, &id).unwrap_or(Ix::default());
        let mut out = Vec::new();
        if size == 0 {
            return out;
        }
        let mut k2 = start_key_.or_else(|| total_len.checked_prev());
        for _ in 0..size {
            let key = match k2 {
                Some(key) => key,
                None => break,
            };
            let full_key = (id, key);
            if let Some(v) = self.store.get(api, &full_key) {
                out.push((key, v));
            }
            k2 = key.checked_prev();
        }
        out
    }

    /// 返回 k1 下全部 (k2, value)。
    pub fn list_all(&self, api: &mut impl Env, k1: &K1) -> Vec<(Ix, V)> {
        let id = match self.k1_to_id.get(api, k1) {
            Some(i) => i,
            None => return Vec::new(),
        };
        let total_len = self.k2_next_id.get(api, &id).unwrap_or(Ix::default());
        let mut out = Vec::new();
        let mut k2 = Ix::default();
        while k2 < total_len {
            let key = (id, k2);
            if let Some(v) = self.store.get(api, &key) {
                out.push((k2, v));
            }
            k2 = match k2.checked_next() {
                Some(n) => n,
                None => break,
            };
        }
        out
    }
}

// ============== 宏：按内层 id 类型定义 List2D 类型别名 ==============

/// 定义内层 id 为 u8 的 List2D。例：`list_2d_u8!(MyDList, K1Ty, ValueTy)` → `type MyDList = List2D<K1Ty, u8, ValueTy>`。
#[macro_export]
macro_rules! list_2d_u8 {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        pub type $name = $crate::List2D<$k1_ty, u8, $value_ty>;
    };
}

/// 定义内层 id 为 u16 的 List2D。
#[macro_export]
macro_rules! list_2d_u16 {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        pub type $name = $crate::List2D<$k1_ty, u16, $value_ty>;
    };
}

/// 定义内层 id 为 u32 的 List2D。
#[macro_export]
macro_rules! list_2d_u32 {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        pub type $name = $crate::List2D<$k1_ty, u32, $value_ty>;
    };
}

/// 定义内层 id 为 u64 的 List2D。
#[macro_export]
macro_rules! list_2d_u64 {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        pub type $name = $crate::List2D<$k1_ty, u64, $value_ty>;
    };
}
