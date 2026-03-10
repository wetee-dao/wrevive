use parity_scale_codec::{Encode, Decode, Output, Input, Error};

/// Trait for types that can be stored in contract storage.
/// Trait for types that can be stored in contract storage.
/// 可以在合约存储中存储的类型的 trait。
/// 
/// # English
/// Defines how a type can be encoded to and decoded from storage.
/// Default implementation uses SCALE codec.
/// 
/// # 中文
/// 定义如何将类型编码到存储以及从存储解码。
/// 默认实现使用 SCALE 编解码。
pub trait Storable: Sized {
    /// Convert self to a slice and append it to the destination.
    /// 将 self 转换为切片并追加到目标。
    /// 
    /// # English
    /// Encodes the type and writes the bytes to the output destination.
    /// 
    /// # 中文
    /// 编码类型并将字节写入输出目标。
    fn encode<T: Output + ?Sized>(&self, dest: &mut T);

    /// Attempt to deserialize the value from input.
    /// 尝试从输入反序列化值。
    /// 
    /// # English
    /// Decodes the type from an input source, returning the result.
    /// 
    /// # 中文
    /// 从输入源解码类型，返回结果。
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error>;

    /// The exact number of bytes this type consumes in the encoded form.
    /// 此类型在编码形式中消耗的确切字节数。
    /// 
    /// # English
    /// Returns the exact size in bytes of the encoded representation.
    /// 
    /// # 中文
    /// 返回编码表示的确切字节大小。
    fn encoded_size(&self) -> usize;
}

impl<T: Encode + Decode> Storable for T {
    fn encode<O: Output + ?Sized>(&self, dest: &mut O) {
        self.encode_to(dest);
    }
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        T::decode(input)
    }
    fn encoded_size(&self) -> usize {
        self.size_hint()
    }
}
