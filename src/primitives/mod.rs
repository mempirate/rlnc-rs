//! RLNC primitives.
pub(crate) mod packet;

use field::Scalar;

use crate::common::SAFE_BYTES_PER_SCALAR;

/// A chunk of data.
#[derive(Debug, Clone)]
pub(crate) struct Chunk {
    symbols: Vec<Scalar>,
}

impl Chunk {
    /// Creates a new chunk from a slice of bytes, and converts it into a vector of scalars
    /// (symbols used for encoding).
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            symbols: bytes
                .chunks(SAFE_BYTES_PER_SCALAR)
                .map(|c| {
                    let mut bytes = [0u8; 32];
                    bytes[..c.len()].copy_from_slice(c);
                    Scalar::from_bytes_le(&bytes).unwrap()
                })
                .collect(),
        }
    }

    /// Returns the symbols of the chunk.
    pub(crate) fn symbols(&self) -> &[Scalar] {
        &self.symbols
    }
}

pub(crate) mod field {
    pub(crate) use blstrs::Scalar;
    pub(crate) use group::ff::Field;
}
