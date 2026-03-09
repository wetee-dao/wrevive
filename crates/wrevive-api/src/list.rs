//! # List
//!
//! Sequential list storage: auto-increment id (K = u8/u16/u32/u64) as key; insert/get/update and paginated list/desc_list.
//! 顺序列表存储：自增 id（K = u8/u16/u32/u64）作为 key，支持 insert/get/update 与分页 list/desc_list。
//! Layout: next_id (Storage<K>) + Mapping<K, V> for items; see primitives define_map. Use list! macro for prefix.

#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;

use crate::env::Env;
use crate::mapping::Mapping;
use crate::Storage;

/// 列表索引类型约束：支持自增/自减与 Scale 编解码，用于 List / List2D 的 id 类型。
pub trait ListIndex: Copy + Default + PartialOrd + parity_scale_codec::Encode + parity_scale_codec::Decode {
    /// 自增 1，溢出返回 None。
    fn checked_next(self) -> Option<Self>;
    /// 自减 1，下溢返回 None。
    fn checked_prev(self) -> Option<Self>;
}

impl ListIndex for u8 {
    fn checked_next(self) -> Option<Self> { self.checked_add(1) }
    fn checked_prev(self) -> Option<Self> { self.checked_sub(1) }
}
impl ListIndex for u16 {
    fn checked_next(self) -> Option<Self> { self.checked_add(1) }
    fn checked_prev(self) -> Option<Self> { self.checked_sub(1) }
}
impl ListIndex for u32 {
    fn checked_next(self) -> Option<Self> { self.checked_add(1) }
    fn checked_prev(self) -> Option<Self> { self.checked_sub(1) }
}
impl ListIndex for u64 {
    fn checked_next(self) -> Option<Self> { self.checked_add(1) }
    fn checked_prev(self) -> Option<Self> { self.checked_sub(1) }
}

/// 列表存储：`next_id` 存当前长度（下一个将分配的 id），`items` 为 id -> value 的映射。K 为 id 类型（u8/u16/u32/u64）。
#[derive(Clone, Copy)]
pub struct List<K, V> {
    next_id: Storage<K>,
    items: Mapping<K, V>,
}

impl<K, V> List<K, V>
where
    K: ListIndex,
    V: parity_scale_codec::Encode + parity_scale_codec::Decode,
{
    /// 使用两个 prefix：一个存 next_id，一个存 items。例如 `List::new(b"mylist_next", b"mylist_i")`。
    pub const fn new(prefix_next_id: &'static [u8], prefix_items: &'static [u8]) -> Self {
        Self {
            next_id: Storage::new(prefix_next_id),
            items: Mapping::new(prefix_items),
        }
    }

    /// 当前长度（即下一个将分配的 id）。
    pub fn len(&self, api: &mut dyn Env) -> K {
        self.next_id.get(api).unwrap_or(K::default())
    }

    /// 插入一条记录，返回分配的 id。
    pub fn insert(&self, api: &mut dyn Env, value: &V) -> Option<K> {
        let k = self.len(api);
        let next = k.checked_next()?;
        self.next_id.set(api, &next);
        self.items.set(api, &k, value);
        Some(k)
    }

    /// 是否存在该 key。
    pub fn contains(&self, api: &mut dyn Env, key: &K) -> bool {
        self.items.get(api, key).is_some()
    }

    /// 按 key 取值。
    pub fn get(&self, api: &mut dyn Env, key: &K) -> Option<V> {
        self.items.get(api, key)
    }

    /// 更新 key 对应的值。
    pub fn update(&self, api: &mut dyn Env, key: &K, value: &V) -> Option<()> {
        self.items.set(api, key, value)
    }

    /// 清除 key 对应的值（不改变 len/next_id）。
    pub fn clear(&self, api: &mut dyn Env, key: &K) -> Option<()> {
        self.items.clear(api, key)
    }

    /// Paginated list (ascending): from start_key, at most size entries.
    /// 分页列表（升序）：从 start_key 起取最多 size 条。
    pub fn list(&self, api: &mut dyn Env, start_key: K, size: u32) -> Vec<(K, V)> {
        let total_len = self.len(api);           // 下一个将分配的 id，即当前长度 / next id = current length
        let mut out = Vec::new();
        if size == 0 {
            return out;
        }
        let mut k = start_key;
        for _ in 0..size {
            if k >= total_len {
                break;                          // 超出范围即停 / stop when beyond range
            }
            if let Some(v) = self.get(api, &k) {
                out.push((k, v));
            }
            k = match k.checked_next() {        // 自增到下一个 id，溢出则结束 / next id, break on overflow
                Some(n) => n,
                None => break,
            };
        }
        out
    }

    /// Paginated list (descending): from start_key_ backward, at most size entries; None = from end.
    /// 分页列表（降序）：从 start_key_ 起向前取最多 size 条；None 表示从末尾开始。
    pub fn desc_list(&self, api: &mut dyn Env, start_key_: Option<K>, size: u32) -> Vec<(K, V)> {
        let total_len = self.len(api);
        let mut out = Vec::new();
        if size == 0 {
            return out;
        }
        // None 时从最后一个有效 id（total_len-1）开始 / when None, start from last valid id
        let mut k = start_key_.or_else(|| total_len.checked_prev());
        for _ in 0..size {
            let key = match k {
                Some(key) => key,
                None => break,                   // 下溢结束 / stop on underflow
            };
            if let Some(v) = self.get(api, &key) {
                out.push((key, v));
            }
            k = key.checked_prev();             // 向前移动 / move backward
        }
        out
    }
}

