//! Field elements.
pub(crate) use blstrs::Scalar;
pub(crate) use group::ff::Field as FiniteField;

/// A field element. This trait inherits from [`ff::Field`](group::ff::Field) and adds methods
/// for converting to and from byte slices.
///
/// The byte size of the field element is specified by the `N` const generic.
pub trait Field: FiniteField {
    /// The maximum number of bytes that can be safely stored in a field element.
    const SAFE_CAPACITY: usize;

    /// Converts a byte slice into a field element.
    fn from_bytes(bytes: &[u8]) -> Self;

    /// Converts a field element into a byte vector.
    fn to_bytes(&self) -> Vec<u8>;
}

impl Field for Scalar {
    const SAFE_CAPACITY: usize = 31;

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut buf = [0u8; 32];
        buf[..bytes.len()].copy_from_slice(bytes);
        Scalar::from_bytes_le(&buf).unwrap()
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes_le()[..Self::SAFE_CAPACITY].to_vec()
    }
}
