//! Module that implements the RLNC encoding algorithm.

use bytes::{Bytes, BytesMut};
use rand::{Rng, rngs::ThreadRng};

use crate::{
    common::{BOUNDARY_MARKER, RLNCError},
    primitives::{galois::GF256, packet::RLNCPacket},
};

#[derive(Debug, Clone)]
pub struct Encoder {
    // The original data to be encoded.
    data: Bytes,
    // The number of chunks to split the data into (also known as the generation size).
    chunk_count: usize,
    // The size of each chunk in bytes.
    chunk_size: usize,
    // The random number generator.
    rng: ThreadRng,
}

impl Encoder {
    /// Creates a new encoder for the given data and chunk count.
    ///
    /// # Arguments
    ///
    /// - `data` - The data to be encoded.
    /// - `chunk_count` - The number of chunks to split the data into (also known as the generation
    ///   size).
    pub fn new(data: impl AsRef<[u8]>, chunk_count: usize) -> Result<Self, RLNCError> {
        if data.as_ref().is_empty() {
            return Err(RLNCError::EmptyData);
        }

        if chunk_count == 0 {
            return Err(RLNCError::ZeroChunkCount);
        }

        let mut data = BytesMut::from(data.as_ref());

        // Add 1 for boundary marker.
        let chunk_size = (data.len() + 1).div_ceil(chunk_count);
        let padded_len = chunk_size * chunk_count;

        // Pad the data with zeros if it's not a multiple of the chunk size.
        data.resize(padded_len, 0);

        // Add boundary marker to the last chunk.
        data[padded_len - 1] = BOUNDARY_MARKER;

        Ok(Self { data: data.freeze(), chunk_count, chunk_size, rng: ThreadRng::default() })
    }

    /// Encodes the data with the given coding vector using linear combinations.
    ///
    /// This method computes a coded packet by taking a linear combination of all chunks
    /// using the coefficients from the coding vector. The operation is performed in
    /// Galois Field GF(256).
    ///
    /// # Mathematical Representation
    ///
    /// Given original chunks X₁, X₂, ..., Xₖ and coding vector coefficients c₁, c₂, ..., cₖ,
    /// the coded packet Y is computed as:
    ///
    /// ```text
    /// Y = c₁ ⊗ X₁ ⊕ c₂ ⊗ X₂ ⊕ ... ⊕ cₖ ⊗ Xₖ
    /// ```
    ///
    /// Where:
    /// - ⊗ denotes multiplication in GF(256)
    /// - ⊕ denotes addition in GF(256) (equivalent to XOR)
    /// - k is the chunk count (generation size)
    ///
    /// Each byte position j in the coded packet is computed as:
    /// ```text
    /// Y[j] = Σᵢ₌₁ᵏ (cᵢ ⊗ Xᵢ[j])  (mod GF(256))
    /// ```
    pub fn encode_with_vector(&self, coding_vector: &[GF256]) -> Result<RLNCPacket, RLNCError> {
        if coding_vector.len() != self.chunk_count {
            return Err(RLNCError::InvalidCodingVectorLength);
        }

        // The result is a vector of GF256 values, one for each byte in the chunk.
        let mut result = vec![GF256::zero(); self.chunk_size];

        // TODO: Optimize this. SIMD? Parallel?
        // First stage: divide the data into chunks
        for (chunk, &coefficient) in self.data.chunks_exact(self.chunk_size).zip(coding_vector) {
            if coefficient == GF256::zero() {
                // Result is zero, skip.
                continue;
            }

            // Second stage: decompose chunks into symbols (GF256 -> u8), and multiply by the
            // coefficient.
            //
            // Y[j] = Σᵢ₌₁ᵏ (cᵢ ⊗ Xᵢ[j])  (mod GF(256))
            for (i, &byte) in chunk.iter().enumerate() {
                result[i] += GF256::from(byte) * coefficient;
            }
        }

        Ok(RLNCPacket { coding_vector: coding_vector.to_vec(), data: result })
    }

    /// Encodes the data with a random coding vector.
    pub fn encode(&self) -> Result<RLNCPacket, RLNCError> {
        let coding_vector =
            self.rng.clone().random_iter().take(self.chunk_count).collect::<Vec<_>>();

        self.encode_with_vector(&coding_vector)
    }

    /// Returns the chunk size used by this encoder.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder() {
        let data = b"Hello, world!";
        let chunk_count = 3;

        let encoder = Encoder::new(data, chunk_count).unwrap();
        println!("{:?}", encoder);

        let packet = encoder.encode().unwrap();

        println!("{:?}", packet);

        assert_eq!(packet.data.len(), encoder.chunk_size);
    }
}
