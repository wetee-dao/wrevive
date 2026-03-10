// Copyright (C) Use Ink (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::Encode;
use parity_scale_codec::{MaxEncodedLen, Output};
use crate::traits::Storable;

    /// A static buffer of variable capacity.
    /// 具有可变容量的静态缓冲区。
    /// 
    /// # English
    /// A fixed-capacity buffer that can be used for temporary storage
    /// without heap allocation. The capacity is determined at compile time.
    /// 
    /// # 中文
    /// 具有固定容量的缓冲区，可用于临时存储
    /// 而无需堆分配。容量在编译时确定。
pub struct StaticBuffer {
    /// A static buffer of variable capacity.
    /// 具有可变容量的静态缓冲区。
    /// 
    /// # English
    /// The underlying byte array with fixed capacity.
    /// 
    /// # 中文
    /// 具有固定容量的底层字节数组。
    buffer: [u8; Self::CAPACITY],
}

impl StaticBuffer {
    /// The capacity of the static buffer.
    /// Usually set to 16 kB.
    /// Can be modified by setting `INK_STATIC_BUFFER_SIZE` environmental variable.
    /// 静态缓冲区的容量。
    /// 通常设置为 16 kB。
    /// 可通过设置 `INK_STATIC_BUFFER_SIZE` 环境变量修改。
    /// 
    /// # English
    /// The total size in bytes of the static buffer.
    /// Defaults to 16384 bytes (16 KB).
    /// Can be overridden via the INK_STATIC_BUFFER_SIZE env var.
    /// 
    /// # 中文
    /// 静态缓冲区的总字节大小。
    /// 默认为 16384 字节（16 KB）。
    /// 可通过 INK_STATIC_BUFFER_SIZE 环境变量覆盖。
    const CAPACITY: usize = crate::BUFFER_SIZE;

    /// Creates a new static buffer.
    /// 创建新的静态缓冲区。
    /// 
    /// # English
    /// Creates a new StaticBuffer instance initialized to all zeros.
    /// 
    /// # 中文
    /// 创建新的 StaticBuffer 实例，初始化为全零。
    pub const fn new() -> Self {
        Self {
            buffer: [0; Self::CAPACITY],
        }
    }
}

impl core::ops::Index<core::ops::RangeFull> for StaticBuffer {
    type Output = [u8];

    #[inline(always)]
    fn index(&self, index: core::ops::RangeFull) -> &Self::Output {
        core::ops::Index::index(&self.buffer[..], index)
    }
}

impl core::ops::IndexMut<core::ops::RangeFull> for StaticBuffer {
    #[inline(always)]
    fn index_mut(&mut self, index: core::ops::RangeFull) -> &mut Self::Output {
        core::ops::IndexMut::index_mut(&mut self.buffer[..], index)
    }
}

    /// Utility to allow for non-heap allocating encoding into a static buffer.
    ///
    /// Required by `ScopedBuffer` internals.
    /// 允许在静态缓冲区中进行非堆分配编码的工具。
    ///
    /// `ScopedBuffer` 内部所需。
    /// 
    /// # English
    /// A scoped encoding context that writes into a borrowed byte slice.
    /// This enables encoding without heap allocations.
    /// 
    /// # 中文
    /// 写入借用的字节切片的作用域编码上下文。
    /// 这使得无需堆分配即可进行编码。
pub struct EncodeScope<'a> {
    buffer: &'a mut [u8],
    len: usize,
}

impl<'a> From<&'a mut [u8]> for EncodeScope<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { buffer, len: 0 }
    }
}

impl<'a> EncodeScope<'a> {
    /// Returns the capacity of the encoded scope.
    /// 返回编码作用域的容量。
    /// 
    /// # English
    /// Returns the total number of bytes the underlying buffer can hold.
    /// 
    /// # 中文
    /// 返回底层缓冲区可容纳的总字节数。
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the length of the encoded scope.
    /// 返回编码作用域的长度。
    /// 
    /// # English
    /// Returns the current number of bytes that have been written.
    /// 
    /// # 中文
    /// 返回已写入的当前字节数。
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the internal mutable byte slice.
    /// 返回内部可变字节切片。
    /// 
    /// # English
    /// Consumes the EncodeScope and returns the underlying mutable buffer.
    /// 
    /// # 中文
    /// 消耗 EncodeScope 并返回底层可变缓冲区。
    pub fn into_buffer(self) -> &'a mut [u8] {
        self.buffer
    }
}

impl Output for EncodeScope<'_> {
    fn write(&mut self, bytes: &[u8]) {
        debug_assert!(
            self.len().checked_add(bytes.len()).unwrap() <= self.capacity(),
            "encode scope buffer overflowed. capacity is {} but last write index is {}",
            self.capacity(),
            self.len().checked_add(bytes.len()).unwrap(),
        );
        let start = self.len;
        let len_bytes = bytes.len();
        self.buffer[start..(start.checked_add(len_bytes)).unwrap()]
            .copy_from_slice(bytes);
        self.len = self.len.checked_add(len_bytes).unwrap();
    }

    fn push_byte(&mut self, byte: u8) {
        debug_assert_ne!(
            self.len(),
            self.capacity(),
            "encode scope buffer overflowed. capacity is {} and buffer is already full",
            self.capacity(),
        );
        self.buffer[self.len] = byte;
        self.len = self.len.checked_add(1).unwrap();
    }
}

    /// Scoped access to an underlying bytes buffer.
    ///
    /// # Note
    ///
    /// This is used to efficiently chunk up ink!'s internal static 16 kB buffer
    /// into smaller sub buffers for processing different parts of computations.
    /// 对底层字节缓冲区的作用域访问。
    ///
    /// # 注意
    ///
    /// 这用于高效地将 ink! 的内部静态 16 kB 缓冲区
    /// 分块为更小的子缓冲区，以处理计算的不同部分。
    /// 
    /// # English
    /// Provides scoped, mutable access to a byte buffer.
    /// Enables efficient chunking of large buffers for different computations.
    /// 
    /// # 中文
    /// 提供对字节缓冲区的作用域化、可变访问。
    /// 支持高效地将大缓冲区分块用于不同计算。
#[derive(Debug)]
pub struct ScopedBuffer<'a> {
    offset: usize,
    buffer: &'a mut [u8],
}

impl<'a> From<&'a mut [u8]> for ScopedBuffer<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { offset: 0, buffer }
    }
}

impl<'a> ScopedBuffer<'a> {
    /// Splits the scoped buffer into yet another piece to operate on it temporarily.
    ///
    /// The split buffer will have an offset of 0 but be offset by `self`'s offset.
    /// 将作用域缓冲区分割为另一部分以临时操作。
    ///
    /// 分割后的缓冲区偏移量为 0，但会受到 `self` 偏移量的影响。
    /// 
    /// # English
    /// Creates a new ScopedBuffer that references the remaining portion
    /// of this buffer's underlying data.
    /// 
    /// # 中文
    /// 创建新的 ScopedBuffer，引用此缓冲区
    /// 底层数据的剩余部分。
    pub fn split(&mut self) -> ScopedBuffer<'_> {
        ScopedBuffer {
            offset: 0,
            buffer: &mut self.buffer[self.offset..],
        }
    }

    /// Returns the first `len` bytes of the buffer as mutable slice.
    /// 返回缓冲区的前 `len` 字节作为可变切片。
    /// 
    /// # English
    /// Takes the first len bytes from the buffer, removing them from
    /// the remaining available space. Returns the taken bytes as a mutable slice.
    /// 
    /// # 中文
    /// 从缓冲区获取前 len 字节，将其从
    /// 剩余可用空间中移除。以可变切片形式返回获取的字节。
    pub fn take(&mut self, len: usize) -> &'a mut [u8] {
        debug_assert_eq!(self.offset, 0);
        debug_assert!(len <= self.buffer.len());
        let len_before = self.buffer.len();
        let buffer = core::mem::take(&mut self.buffer);
        let (lhs, rhs) = buffer.split_at_mut(len);
        self.buffer = rhs;
        debug_assert_eq!(lhs.len(), len);
        let len_after = self.buffer.len();
        debug_assert_eq!(len_before.checked_sub(len_after).unwrap(), len);
        lhs
    }

    /// Returns the first [`MaxEncodedLen::max_encoded_len`] bytes of the buffer as
    /// a mutable slice.
    /// 返回缓冲区的前 [`MaxEncodedLen::max_encoded_len`] 字节作为可变切片。
    /// 
    /// # English
    /// Takes the maximum number of bytes needed to encode type T from the buffer.
    /// 
    /// # 中文
    /// 从缓冲区获取编码类型 T 所需的最大字节数。
    #[inline(always)]
    pub fn take_max_encoded_len<T>(&mut self) -> &'a mut [u8]
    where
        T: MaxEncodedLen,
    {
        let len = T::max_encoded_len();
        self.take(len)
    }

    /// Encodes the given value into the scoped buffer and returns the sub slice
    /// containing all the encoded bytes.
    /// 将给定值编码到作用域缓冲区并返回包含所有编码字节的子切片。
    /// 
    /// # English
    /// Encodes value using SCALE codec into the buffer and returns
    /// the slice containing the encoded data.
    /// 
    /// # 中文
    /// 使用 SCALE 编解码将值编码到缓冲区并返回
    /// 包含编码数据的切片。
    #[inline(always)]
    pub fn take_encoded<T>(&mut self, value: &T) -> &'a mut [u8]
    where
        T: Encode,
    {
        debug_assert_eq!(self.offset, 0);
        let buffer = core::mem::take(&mut self.buffer);
        let mut encode_scope = EncodeScope::from(buffer);
        Encode::encode_to(value, &mut encode_scope);
        let encode_len = encode_scope.len();
        let _ = core::mem::replace(&mut self.buffer, encode_scope.into_buffer());
        self.take(encode_len)
    }

    /// Encodes the given value into the scoped buffer using the given encoder function
    /// and returns the sub slice containing all the encoded bytes.
    /// 使用给定的编码器函数将值编码到作用域缓冲区并返回包含所有编码字节的子切片。
    /// 
    /// # English
    /// Uses a custom encoder function to write bytes into the buffer
    /// and returns the slice containing the encoded data.
    /// 
    /// # 中文
    /// 使用自定义编码器函数将字节写入缓冲区
    /// 并返回包含编码数据的切片。
    pub fn take_encoded_with<F>(&mut self, encoder: F) -> &'a mut [u8]
    where
        F: FnOnce(&mut [u8]) -> usize,
    {
        let encode_len = encoder(self.buffer);
        self.take(encode_len)
    }

    /// Encodes the given storable value into the scoped buffer and returns the sub slice
    /// containing all the encoded bytes.
    /// 将给定可存储值编码到作用域缓冲区并返回包含所有编码字节的子切片。
    /// 
    /// # English
    /// Encodes a Storable value into the buffer using its custom encoding
    /// and returns the slice containing the encoded data.
    /// 
    /// # 中文
    /// 使用其自定义编码将可存储值编码到缓冲区并返回
    /// 包含编码数据的切片。
    #[inline(always)]
    pub fn take_storable_encoded<T>(&mut self, value: &T) -> &'a mut [u8]
    where
        T: Storable,
    {
        debug_assert_eq!(self.offset, 0);
        let buffer = core::mem::take(&mut self.buffer);
        let mut encode_scope = EncodeScope::from(buffer);
        Storable::encode(value, &mut encode_scope);
        let encode_len = encode_scope.len();
        let _ = core::mem::replace(&mut self.buffer, encode_scope.into_buffer());
        self.take(encode_len)
    }

    /// Appends the given bytes to the scoped buffer.
    ///
    /// Does not return the buffer immediately so that other values can be appended
    /// afterward. The [`take_appended`] method shall be used to return the buffer
    /// that includes all appended encodings as a single buffer.
    /// 将给定字节追加到作用域缓冲区。
    ///
    /// 不会立即返回缓冲区，以便后续可以追加其他值。
    /// 应使用 [`take_appended`] 方法返回包含所有追加编码的单个缓冲区。
    /// 
    /// # English
    /// Appends bytes to the end of the current write position.
    /// The offset advances but the buffer is not returned yet.
    /// 
    /// # 中文
    /// 将字节追加到当前写入位置的末尾。
    /// 偏移量前进但缓冲区尚未返回。
    #[inline(always)]
    pub fn append_bytes(&mut self, bytes: &[u8]) {
        let offset = self.offset;
        let len = bytes.len();
        let end_offset = offset.checked_add(len).unwrap();
        self.buffer[offset..end_offset].copy_from_slice(bytes);
        self.offset = end_offset;
    }

    /// Returns the buffer containing all encodings appended via [`append_encoded`]
    /// in a single byte buffer.
    /// 返回包含通过 [`append_encoded`] 追加的所有编码的单字节缓冲区。
    /// 
    /// # English
    /// Returns all bytes that have been appended since the last reset.
    /// Resets the offset for future appends.
    /// 
    /// # 中文
    /// 返回自上次重置以来已追加的所有字节。
    /// 重置偏移量以备将来追加。
    pub fn take_appended(&mut self) -> &'a mut [u8] {
        debug_assert_ne!(self.offset, 0);
        let offset = self.offset;
        self.offset = 0;
        self.take(offset)
    }

    /// Returns all remaining bytes of the buffer as a mutable slice.
    /// 返回缓冲区所有剩余字节作为可变切片。
    /// 
    /// # English
    /// Consumes the ScopedBuffer and returns all remaining bytes.
    /// 
    /// # 中文
    /// 消耗 ScopedBuffer 并返回所有剩余字节。
    pub fn take_rest(self) -> &'a mut [u8] {
        debug_assert_eq!(self.offset, 0);
        debug_assert!(!self.buffer.is_empty());
        self.buffer
    }

    /// Returns the size of all remaining bytes in the buffer.
    ///
    /// # Developer Note
    ///
    /// The function requires `&mut self` because `self.buffer` is
    /// already a mutable reference in the struct.
    ///
    /// _The function does not actually mutate state._
    /// 返回缓冲区中所有剩余字节的大小。
    ///
    /// # 开发者注意
    ///
    /// 该函数需要 `&mut self`，因为 `self.buffer`
    /// 已经是结构体中的可变引用。
    ///
    /// _该函数实际上不改变状态。_
    /// 
    /// # English
    /// Returns the number of bytes remaining in the buffer.
    /// Requires mutable self for borrowing reasons, but does not modify state.
    /// 
    /// # 中文
    /// 返回缓冲区中剩余的字节数。
    /// 出于借用原因需要可变 self，但不修改状态。
    pub fn remaining_buffer(&mut self) -> usize {
        self.buffer.len()
    }
}