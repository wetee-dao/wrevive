//! # List
//!
//! Sequential list storage: auto-increment id (K = u8/u16/u32/u64) as key.
//! Supports insert/get/update and paginated list/desc_list.
//! 顺序列表存储：自增 id（K = u8/u16/u32/u64）作为 key。
//! 支持 insert/get/update 与分页 list/desc_list。
//!
//! Layout: next_id (Storage<K>) + Mapping<K, V> for items.
//! 布局：next_id (Storage<K>) + Mapping<K, V> 存储条目。

#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;
use crate::mapping::Mapping;
use crate::Storage;

/// List index type constraint: supports increment/decrement and SCALE codec.
/// Used for List/List2D id types.
/// 列表索引类型约束：支持自增/自减与 SCALE 编解码。
/// 用于 List/List2D 的 id 类型。
pub trait ListIndex: Copy + Default + PartialOrd + parity_scale_codec::Encode + parity_scale_codec::Decode {
    /// Increments by 1, returns None on overflow.
    /// 自增 1，溢出返回 None。
    fn checked_next(self) -> Option<Self>;
    /// Decrements by 1, returns None on underflow.
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

/// List storage: `next_id` stores current length (next id to be assigned),
/// `items` is the mapping from id to value. K is the id type (u8/u16/u32/u64).
/// 列表存储：`next_id` 存当前长度（下一个将分配的 id），`items` 为 id -> value 的映射。
/// K 为 id 类型（u8/u16/u32/u64）。
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
    /// Creates with two prefixes: one for next_id, one for items.
    /// 使用两个 prefix：一个存 next_id，一个存 items。
    ///
    /// Example: `List::new(b"mylist_next", b"mylist_i")`
    /// 例如：`List::new(b"mylist_next", b"mylist_i")`
    pub const fn new(prefix_next_id: &'static [u8], prefix_items: &'static [u8]) -> Self {
        Self {
            next_id: Storage::new(prefix_next_id),
            items: Mapping::new(prefix_items),
        }
    }

    /// Current length (i.e., next id to be assigned).
    /// 当前长度（即下一个将分配的 id）。
    pub fn len(&self) -> K {
        self.next_id.get().unwrap_or(K::default())
    }

    /// Inserts a record and returns the assigned id.
    /// 插入一条记录，返回分配的 id。
    pub fn insert(&self, value: &V) -> Option<K> {
        let k = self.len();
        let next = k.checked_next()?;
        self.next_id.set(&next);
        self.items.set(&k, value);
        Some(k)
    }

    /// Checks if the key exists.
    /// 是否存在该 key。
    pub fn contains(&self, key: &K) -> bool {
        self.items.get(key).is_some()
    }

    /// Gets value by key.
    /// 按 key 取值。
    pub fn get(&self, key: &K) -> Option<V> {
        self.items.get(key)
    }

    /// Updates the value for the given key.
    /// 更新 key 对应的值。
    pub fn update(&self, key: &K, value: &V) -> Option<()> {
        self.items.set(key, value)
    }

    /// Clears the value for the given key (does not change len/next_id).
    /// 清除 key 对应的值（不改变 len/next_id）。
    pub fn clear(&self, key: &K) -> Option<()> {
        self.items.clear(key)
    }

    /// Paginated list (ascending): from start_key, at most size entries.
    /// 分页列表（升序）：从 start_key 起取最多 size 条。
    pub fn list(&self, start_key: K, size: u32) -> Vec<(K, V)> {
        let total_len = self.len(); // next id = current length
        let mut out = Vec::new();
        if size == 0 {
            return out;
        }
        let mut k = start_key;
        for _ in 0..size {
            if k >= total_len {
                break;                          // 超出范围即停 / stop when beyond range
            }
            if let Some(v) = self.get(&k) {
                out.push((k, v));
            }
            k = match k.checked_next() { // next id, break on overflow
                Some(n) => n,
                None => break,
            };
        }
        out
    }

    /// Paginated list (descending): from start_key_ backward, at most size entries.
    /// 分页列表（降序）：从 start_key_ 起向前取最多 size 条。
    ///
    /// None means starting from the end.
    /// None 表示从末尾开始。
    pub fn desc_list(&self, start_key_: Option<K>, size: u32) -> Vec<(K, V)> {
        let total_len = self.len();
        let mut out = Vec::new();
        if size == 0 {
            return out;
        }
        // when None, start from last valid id (total_len-1)
        let mut k = start_key_.or_else(|| total_len.checked_prev());
        for _ in 0..size {
            let key = match k {
                Some(key) => key,
                None => break,                   // 下溢结束 / stop on underflow
            };
            if let Some(v) = self.get(&key) {
                out.push((key, v));
            }
            k = key.checked_prev(); // move backward
        }
        out
    }
}

