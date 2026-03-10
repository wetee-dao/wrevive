//! # List2D
//!
//! Two-dimensional list: grouped by K1, each group has an auto-increment Ix (u8/u16/u32/u64) index.
//! 二维列表：按 K1 分组，每组内为自增 Ix 索引的列表。
//!
//! Layout: k1->id, k1_length, k2_next_id per id, store (id, k2)->value.
//! 布局：k1->id, k1_length, k2_next_id per id, store (id, k2)->value。

#[cfg(not(any(test, feature = "off_chain")))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "off_chain"))]
use std::vec::Vec;
use crate::list::ListIndex;
use crate::mapping::Mapping;
use crate::Storage;

/// Two-dimensional list: outer key K1, inner auto-increment index Ix (u8/u16/u32/u64).
/// When K1 first appears, an id is assigned; under this id, k2 increments from 0.
/// 二维列表：外层 key K1，内层自增索引 Ix（u8/u16/u32/u64）。
/// K1 首次出现时分配一个 id，该 id 下 k2 从 0 自增。
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
    /// Creates a new List2D instance with four prefixes for internal storage.
    /// 创建具有四个前缀的新 List2D 实例。
    ///
    /// Prefixes:
    /// - k1_to_id: maps outer keys (K1) to internal IDs / 将外层键 (K1) 映射到内部 ID
    /// - k1_length: tracks number of distinct K1 keys / 跟踪不同 K1 键的数量
    /// - k2_next_id: maps internal IDs to next available inner index / 将内部 ID 映射到下一个可用的内层索引
    /// - store: stores (inner_id, inner_index) -> value / 存储 (内层_id, 内层索引) -> 值
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

    /// Returns the next k2 that will be allocated under this K1 (i.e., current length).
    /// 返回该 K1 下下一个将分配的 k2（即当前长度）。
    pub fn next_id(&self, k1: &K1) -> Ix {
        let id = match self.k1_to_id.get(k1) {
            Some(id) => id,
            None => return Ix::default(),
        };
        self.k2_next_id.get(&id).unwrap_or(Ix::default())
    }

    /// Returns the number of entries under this K1 (same as next_id).
    /// 返回该 K1 下的条目数量（与 next_id 一致）。
    pub fn len(&self, k1: &K1) -> Ix {
        self.next_id(k1)
    }

    /// Inserts one entry under k1; returns the allocated k2.
    /// 在 k1 下插入一条记录，返回分配的 k2。
    ///
    /// Returns None if the inner index would overflow (e.g., u8 reaches 256).
    /// 如果内层索引会溢出（例如 u8 达到 256），则返回 None。
    pub fn insert(&self, k1: &K1, value: &V) -> Option<Ix> {
        let mut id = self.k1_to_id.get(k1);
        if id.is_none() {
            let len = self.k1_length.get().unwrap_or(Ix::default());
            id = Some(len);
            self.k1_to_id.set(k1, &len);
            let next_len = len.checked_next()?;
            self.k1_length.set(&next_len);
        }
        let id_val = id?;
        let next_id = self.k2_next_id.get(&id_val).unwrap_or(Ix::default());
        let new_next_id = next_id.checked_next()?;
        self.k2_next_id.set(&id_val, &new_next_id);
        let key = (id_val, next_id);
        self.store.set(&key, value);
        Some(next_id)
    }

    /// Updates the value at (k1, k2).
    /// 更新 (k1, k2) 对应的值。
    ///
    /// Returns None if either k1 does not exist or k2 is out of range for that k1.
    /// 如果 k1 不存在或 k2 超出该 k1 的范围，则返回 None。
    pub fn update(&self, k1: &K1, k2: Ix, value: &V) -> Option<()> {
        let id = self.k1_to_id.get(k1)?;
        let key = (id, k2);
        self.store.set(&key, value)
    }

    /// Clears the value at (k1, k2) without changing the k1's len/next_id.
    /// 清除 (k1, k2) 对应的值（不改变该 k1 的 len/next_id）。
    ///
    /// Note: this does not decrement the length or next_id for k1;
    /// the index remains allocated but holds no value.
    /// 注意：这不会减少 k1 的长度或 next_id；索引仍被分配但不保存值。
    pub fn clear(&self, k1: &K1, k2: Ix) -> Option<()> {
        let id = self.k1_to_id.get(k1)?;
        let key = (id, k2);
        self.store.clear(&key)
    }

    /// Retrieves the value at (k1, k2).
    /// 按 (k1, k2) 取值。
    /// 
    /// # English
    /// Gets the stored value for the given (k1, k2) pair.
    /// Returns None if the pair does not exist or has been cleared.
    /// 
    /// # 中文
    /// 获取给定 (k1, k2) 对的存储值。
    /// 如果该对不存在或已被清除，则返回 None。
    pub fn get(&self, k1: &K1, k2: Ix) -> Option<V> {
        let id = self.k1_to_id.get(k1)?;
        let key = (id, k2);
        self.store.get(&key)
    }

    /// Paginated list in ascending order: up to size entries from k1 starting at start_key.
    /// 分页列表（升序）：k1 下从 start_key 起取最多 size 条。
    /// 
    /// # English
    /// Returns a paginated list of (index, value) pairs for the given k1 in ascending order.
    /// Starts from start_key and returns at most size entries.
    /// Stops early if reaching the end or if an index has been cleared.
    /// 
    /// # 中文
    /// 返回给定 k1 的升序分页 (索引, 值) 对列表。
    /// 从 start_key 开始，最多返回 size 条记录。
    /// 到达末尾或索引已被清除时提前停止。
    pub fn list(&self, k1: &K1, start_key: Ix, size: u32) -> Vec<(Ix, V)> {
        let id = match self.k1_to_id.get(k1) {
            Some(i) => i,
            None => return Vec::new(),
        };
        let total_len = self.k2_next_id.get(&id).unwrap_or(Ix::default());
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
            if let Some(v) = self.store.get(&key) {
                out.push((k2, v));
            }
            k2 = match k2.checked_next() {
                Some(n) => n,
                None => break,
            };
        }
        out
    }

    /// Paginated list in descending order: up to size entries from k1, starting before start_key_; None means from the end.
    /// 分页列表（降序）：k1 下从 start_key_ 起向前取最多 size 条；None 表示从末尾开始。
    /// 
    /// # English
    /// Returns a paginated list of (index, value) pairs for the given k1 in descending order.
    /// If start_key_ is None, starts from the last valid index.
    /// Returns at most size entries, stopping early if indices are exhausted.
    /// 
    /// # 中文
    /// 返回给定 k1 的降序分页 (索引, 值) 对列表。
    /// 如果 start_key_ 为 None，则从最后一个有效索引开始。
    /// 最多返回 size 条记录，索引耗尽时提前停止。
    pub fn desc_list(
        &self,
        k1: &K1,
        start_key_: Option<Ix>,
        size: u32,
    ) -> Vec<(Ix, V)> {
        let id = match self.k1_to_id.get(k1) {
            Some(i) => i,
            None => return Vec::new(),
        };
        let total_len = self.k2_next_id.get(&id).unwrap_or(Ix::default());
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
            if let Some(v) = self.store.get(&full_key) {
                out.push((key, v));
            }
            k2 = key.checked_prev();
        }
        out
    }

    /// Returns all (k2, value) pairs under k1.
    /// 返回 k1 下全部 (k2, value)。
    /// 
    /// # English
    /// Gets all stored (index, value) pairs for the given k1 in ascending order.
    /// Skips indices that have been cleared.
    /// 
    /// # 中文
    /// 按升序获取给定 k1 的所有存储 (索引, 值) 对。
    /// 跳过已被清除的索引。
    pub fn list_all(&self, k1: &K1) -> Vec<(Ix, V)> {
        let id = match self.k1_to_id.get(k1) {
            Some(i) => i,
            None => return Vec::new(),
        };
        let total_len = self.k2_next_id.get(&id).unwrap_or(Ix::default());
        let mut out = Vec::new();
        let mut k2 = Ix::default();
        while k2 < total_len {
            let key = (id, k2);
            if let Some(v) = self.store.get(&key) {
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

/// Defines a List2D with inner index type u8.
/// 定义内层 id 为 u8 的 List2D。
/// 
/// # English
/// Creates a type alias for a List2D with u8 as the inner index type.
/// Example: `list_2d_u8!(MyDList, K1Ty, ValueTy)` expands to `type MyDList = List2D<K1Ty, u8, ValueTy>`
/// 
/// # 中文
/// 创建内层索引类型为 u8 的 List2D 类型别名。
/// 示例：`list_2d_u8!(MyDList, K1Ty, ValueTy)` 展开为 `type MyDList = List2D<K1Ty, u8, ValueTy>`
#[macro_export]
macro_rules! list_2d_u8 {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        pub type $name = $crate::List2D<$k1_ty, u8, $value_ty>;
    };
}

/// Defines a List2D with inner index type u16.
/// 定义内层 id 为 u16 的 List2D。
/// 
/// # English
/// Creates a type alias for a List2D with u16 as the inner index type.
/// 
/// # 中文
/// 创建内层索引类型为 u16 的 List2D 类型别名。
#[macro_export]
macro_rules! list_2d_u16 {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        pub type $name = $crate::List2D<$k1_ty, u16, $value_ty>;
    };
}

/// Defines a List2D with inner index type u32.
/// 定义内层 id 为 u32 的 List2D。
/// 
/// # English
/// Creates a type alias for a List2D with u32 as the inner index type.
/// 
/// # 中文
/// 创建内层索引类型为 u32 的 List2D 类型别名。
#[macro_export]
macro_rules! list_2d_u32 {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        pub type $name = $crate::List2D<$k1_ty, u32, $value_ty>;
    };
}

/// Defines a List2D with inner index type u64.
/// 定义内层 id 为 u64 的 List2D。
/// 
/// # English
/// Creates a type alias for a List2D with u64 as the inner index type.
/// 
/// # 中文
/// 创建内层索引类型为 u64 的 List2D 类型别名。
#[macro_export]
macro_rules! list_2d_u64 {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        pub type $name = $crate::List2D<$k1_ty, u64, $value_ty>;
    };
}
