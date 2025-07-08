//! Module that implements the RLNC encoding algorithm.

use rand::Rng;

use crate::{
    common::{BOUNDARY_MARKER, RLNCError, SAFE_BYTES_PER_SCALAR},
    primitives::{
        Chunk,
        field::{Field, Scalar},
        packet::RLNCPacket,
    },
};

/// Helper function that encodes the data into `chunk_count` packets with random coding vectors, and
/// returns a vector of encoded packets.
pub fn encode(data: &[u8], chunk_count: usize) -> Result<Vec<RLNCPacket>, RLNCError> {
    let encoder = Encoder::new(data, chunk_count)?;

    let mut rng = rand::rng();
    let mut packets = Vec::with_capacity(chunk_count);
    for _ in 0..chunk_count {
        packets.push(encoder.encode(&mut rng)?);
    }

    Ok(packets)
}

/// RLNC Encoder.
#[derive(Debug)]
pub struct Encoder {
    // The chunks of data to be encoded.
    chunks: Vec<Chunk>,
    // The number of chunks to split the data into (also known as the generation size).
    chunk_count: usize,
    // The size of each chunk in bytes.
    chunk_size: usize,
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

        let mut data = Vec::from(data.as_ref());
        data.push(BOUNDARY_MARKER);

        // Calculate chunk size to accommodate original data + boundary marker
        let chunk_size = data.len().div_ceil(chunk_count);

        // Round up chunk size to nearest multiple of 31 for scalar packing
        let chunk_size = chunk_size.div_ceil(SAFE_BYTES_PER_SCALAR) * SAFE_BYTES_PER_SCALAR;
        let padded_len = chunk_size * chunk_count;

        // Pad the rest with zeros if needed
        data.resize(padded_len, 0);

        let chunks = data.chunks_exact(chunk_size).map(Chunk::from_bytes).collect();

        Ok(Self { chunks, chunk_count, chunk_size })
    }

    /// Returns the number of chunks the data was split into.
    pub fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    /// Returns the size of each chunk in bytes.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Encodes the data with the given coding vector using linear combinations.
    ///
    /// This method computes a coded packet by taking a linear combination of all chunks
    /// using the coefficients from the coding vector. The operation is performed in
    /// the field of BLS12-381.
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
    /// - ⊗ denotes multiplication in the field of BLS12-381
    /// - ⊕ denotes addition in the field of BLS12-381
    /// - k is the chunk count (generation size)
    ///
    /// Each byte position j in the coded packet is computed as:
    /// ```text
    /// Y[j] = Σᵢ₌₁ᵏ (cᵢ ⊗ Xᵢ[j])  (mod GF(256))
    /// ```
    ///
    /// # Algorithm Complexity
    /// O(k * n) where k is the chunk count and n is the chunk size.
    /// ```
    pub fn encode_with_vector(&self, coding_vector: &[Scalar]) -> Result<RLNCPacket, RLNCError> {
        if coding_vector.len() != self.chunk_count {
            return Err(RLNCError::InvalidCodingVectorLength(coding_vector.len(), self.chunk_count));
        }

        // The result is a vector of Scalar values, one for each byte in the chunk.
        let mut result = vec![Scalar::ZERO; self.chunk_size.div_ceil(SAFE_BYTES_PER_SCALAR)];

        // TODO: Optimize this. SIMD? Parallel?
        // - https://ssrc.us/media/pubs/c9a735170a7e1aa648b261ec6ad615e34af566db.pdf
        // - https://github.com/geky/gf256?tab=readme-ov-file#hardware-support
        // - https://github.com/AndersTrier/reed-solomon-simd
        // First stage: divide the data into chunks
        for (chunk, &coefficient) in self.chunks.iter().zip(coding_vector) {
            if coefficient.is_zero_vartime() {
                // Result is zero, skip.
                continue;
            }

            // Second stage: decompose chunks into symbols and perform
            // element-wise multiplication with the coefficient.
            //
            // Y[j] = Σᵢ₌₁ᵏ (cᵢ ⊗ Xᵢ[j])
            for (i, symbol) in chunk.scalars().iter().enumerate() {
                result[i] += symbol * coefficient;
            }
        }

        Ok(RLNCPacket { coding_vector: coding_vector.to_vec(), data: result })
    }

    /// Encodes the data with a random coding vector, using the provided random number generator.
    pub fn encode<R: Rng>(&self, mut rng: R) -> Result<RLNCPacket, RLNCError> {
        let coding_vector: Vec<Scalar> = (0..self.chunk_count)
            .map(|_| {
                let mut bytes = [0u8; 32];
                rng.fill(&mut bytes[..SAFE_BYTES_PER_SCALAR]);
                Scalar::from_bytes_le(&bytes).unwrap()
            })
            .collect();

        self.encode_with_vector(&coding_vector)
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

        let packet = encoder.encode(rand::rng()).unwrap();

        println!("{:?}", packet);

        assert_eq!(packet.data.len(), encoder.chunk_size.div_ceil(SAFE_BYTES_PER_SCALAR));
    }
}
