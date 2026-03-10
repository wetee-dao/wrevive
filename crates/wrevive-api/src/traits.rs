use parity_scale_codec::{Encode, Decode, Output, Input, Error};

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
    /// Encodes the type and writes the bytes to the output destination.
    /// 编码类型并将字节写入输出目标。
    fn encode<T: Output + ?Sized>(&self, dest: &mut T);

    /// Decodes the type from an input source, returning the result.
    /// 从输入源解码类型，返回结果。
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error>;

    /// Returns the exact size in bytes of the encoded representation.
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
