pub(crate) mod packet;

use field::Scalar;

use crate::common::SAFE_BYTES_PER_SCALAR;

/// A chunk of data.
#[derive(Debug, Clone)]
pub struct Chunk {
    bytes: Vec<u8>,
    scalars: Vec<Scalar>,
}

impl Chunk {
    /// Creates a new chunk from a slice of bytes.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
            scalars: bytes
                .chunks(SAFE_BYTES_PER_SCALAR)
                .map(|c| {
                    let mut bytes = [0u8; 32];
                    bytes[..c.len()].copy_from_slice(c);
                    Scalar::from_bytes_le(&bytes).unwrap()
                })
                .collect(),
        }
    }

    /// Converts the chunk into a vector of scalars.
    pub(crate) fn scalars(&self) -> &[Scalar] {
        &self.scalars
    }
}

pub(crate) mod field {
    pub(crate) use blstrs::Scalar;
    pub(crate) use group::ff::Field;
}
