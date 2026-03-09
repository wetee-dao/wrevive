use parity_scale_codec::{Encode, Decode, Output, Input, Error};

pub trait Storable: Sized {
    /// Convert self to a slice and append it to the destination.
    fn encode<T: Output + ?Sized>(&self, dest: &mut T);

    /// Attempt to deserialize the value from input.
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error>;

    /// The exact number of bytes this type consumes in the encoded form.
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
