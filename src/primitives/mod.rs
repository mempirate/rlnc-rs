//! RLNC primitives.
pub(crate) mod packet;

use field::Scalar;

use crate::common::SAFE_BYTES_PER_SCALAR;

/// A chunk of data.
#[derive(Debug, Clone)]
pub struct Chunk<S> {
    symbols: Vec<S>,
    size: usize,
}

impl Chunk<Scalar> {
    /// Creates a new chunk from a slice of bytes, and converts it into a vector of scalars
    /// (symbols used for encoding).
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let size = bytes.len();
        Self {
            symbols: bytes
                .chunks(SAFE_BYTES_PER_SCALAR)
                .map(|c| {
                    let mut bytes = [0u8; 32];
                    bytes[..c.len()].copy_from_slice(c);
                    Scalar::from_bytes_le(&bytes).unwrap()
                })
                .collect(),
            size,
        }
    }

    /// Returns the symbols of the chunk.
    pub(crate) fn symbols(&self) -> &[Scalar] {
        &self.symbols
    }

    /// Returns the size of the chunk in bytes.
    pub(crate) fn size(&self) -> usize {
        self.size
    }
}

// TODO: Add a generic implementation for Chunk<S>. In its current form, we have to convert bytes to
// Symbols, but since ff::Field doesn't implement TryFrom<&[u8]>, we can't make it generic.

pub(crate) mod field {
    pub(crate) use blstrs::Scalar;
    pub(crate) use group::ff::Field;
}
