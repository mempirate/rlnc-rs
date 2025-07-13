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

/// RLNC encoder trait.
pub trait Encoder {
    /// The type of the coded data.
    type Codeword;
    /// The type of the symbol used in this encoder. The original data
    /// will be converted to symbols, and the random coding vector will also
    /// be of this type.
    type Symbol: group::ff::Field;
    /// Error type.
    type Error;

    /// Prepares the data for encoding. Divides the data into equally sized chunks, using padding
    /// where necessary. Each chunk is converted into a vector of symbols and returned in
    /// [`Chunk`]s.
    fn prepare(
        data: impl AsRef<[u8]>,
        chunk_count: usize,
    ) -> Result<Vec<Chunk<Self::Symbol>>, RLNCError>;

    /// Creates a new encoder from the given chunks. These chunks should be prepared with
    /// [`Self::prepare`].
    fn from_chunks(chunks: Vec<Chunk<Self::Symbol>>) -> Self;

    /// Returns the chunk count.
    fn chunk_count(&self) -> usize;

    /// Returns the chunk size.
    fn chunk_size(&self) -> usize;

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
    /// Y[j] = Σᵢ₌₁ᵏ (cᵢ ⊗ Xᵢ[j])  (mod p)
    /// ```
    ///
    /// # Algorithm Complexity
    /// O(k * n) where k is the chunk count and n is the chunk size.
    /// ```
    fn encode_with_vector(
        &self,
        coding_vector: &[Self::Symbol],
    ) -> Result<Self::Codeword, Self::Error>;

    /// Encodes the data with a random coding vector, using the provided random number generator.
    fn encode<R: Rng>(&self, rng: R) -> Result<Self::Codeword, Self::Error>;
}

/// An RLNC encoder that commits to original data chunks with non-hiding Pedersen commitments before
/// encoding. It uses [`PedersenCommitter`] to commit to the data chunks.
#[derive(Debug)]
pub struct SecureEncoder {
    // The chunks of data to be encoded.
    chunks: Vec<Chunk<Scalar>>,
    // The number of chunks to split the data into (also known as the generation size).
    chunk_count: usize,
    // The size of each chunk in bytes.
    chunk_size: usize,
}

impl SecureEncoder {
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

        // Round up chunk size to nearest multiple of `SAFE_BYTES_PER_SCALAR` for scalar packing
        let chunk_size = chunk_size.div_ceil(SAFE_BYTES_PER_SCALAR) * SAFE_BYTES_PER_SCALAR;
        let padded_len = chunk_size * chunk_count;

        // Pad the rest with zeros if needed
        data.resize(padded_len, 0);

        let chunks = data.chunks_exact(chunk_size).map(Chunk::from_bytes).collect();

        Ok(Self { chunks, chunk_count, chunk_size })
    }

    /// Returns true if the encoder should parallelize the encoding process.
    ///
    /// This is determined by the chunk count (collection size), chunk size (work unit size), and
    /// the number of threads.
    #[cfg(feature = "parallel")]
    fn should_parallelize(&self) -> bool {
        // Min total work: 512KiB
        let min_total_work = 1024 * 512;
        // Min chunks: 2
        let min_chunks = 2;
        // Min work unit: 128KiB
        let min_work_unit = 1024 * 128;

        let total_work = self.chunk_count * self.chunk_size;

        total_work >= min_total_work &&
            self.chunk_size >= min_work_unit &&
            self.chunk_count >= min_chunks
    }

    /// Sequentially encodes the data with the given coding vector using linear combinations.
    fn encode_inner(&self, coding_vector: &[Scalar]) -> Vec<Scalar> {
        let mut result = vec![Scalar::ZERO; self.chunk_size.div_ceil(SAFE_BYTES_PER_SCALAR)];

        for (chunk, &coefficient) in self.chunks.iter().zip(coding_vector) {
            if coefficient.is_zero_vartime() {
                continue;
            }

            for (i, symbol) in chunk.symbols().iter().enumerate() {
                result[i] += symbol * coefficient;
            }
        }

        result
    }
}

impl Encoder for SecureEncoder {
    type Codeword = RLNCPacket;
    type Symbol = Scalar;

    type Error = RLNCError;

    fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn prepare(
        data: impl AsRef<[u8]>,
        chunk_count: usize,
    ) -> Result<Vec<Chunk<Scalar>>, RLNCError> {
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

        // Round up chunk size to nearest multiple of `SAFE_BYTES_PER_SCALAR` for scalar packing
        let chunk_size = chunk_size.div_ceil(SAFE_BYTES_PER_SCALAR) * SAFE_BYTES_PER_SCALAR;
        let padded_len = chunk_size * chunk_count;

        // Pad the rest with zeros if needed
        data.resize(padded_len, 0);

        Ok(data.chunks_exact(chunk_size).map(Chunk::from_bytes).collect())
    }

    fn from_chunks(chunks: Vec<Chunk<Self::Symbol>>) -> Self {
        let chunk_count = chunks.len();
        let chunk_size = chunks[0].size();

        Self { chunks, chunk_count, chunk_size }
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
    /// Y[j] = Σᵢ₌₁ᵏ (cᵢ ⊗ Xᵢ[j])  (mod p)
    /// ```
    ///
    /// # Algorithm Complexity
    /// O(k * n) where k is the chunk count and n is the chunk size.
    /// ```
    fn encode_with_vector(&self, coding_vector: &[Scalar]) -> Result<RLNCPacket, RLNCError> {
        if coding_vector.len() != self.chunk_count {
            return Err(RLNCError::InvalidCodingVectorLength(coding_vector.len(), self.chunk_count));
        }

        let symbol_count = self.chunk_size.div_ceil(SAFE_BYTES_PER_SCALAR);

        // Compute the encoded result either sequentially or in parallel, depending on the
        // enabled feature flag. We avoid sharing mutable state across threads by letting each
        // worker produce a partial vector and then combining (reducing) the partial results.
        #[cfg(feature = "parallel")]
        let result = {
            use rayon::prelude::*;

            if !self.should_parallelize() {
                self.encode_inner(coding_vector)
            } else {
                // Map each (chunk, coefficient) pair to its contribution and then reduce all
                // contributions into the final result.
                self.chunks
                    .par_iter()
                    .zip(coding_vector)
                    .filter_map(|(chunk, &coefficient)| {
                        // Skip the work if the coefficient is zero.
                        if coefficient.is_zero_vartime() {
                            return None;
                        }

                        let mut acc = Vec::with_capacity(symbol_count);

                        for symbol in chunk.symbols().iter() {
                            acc.push(*symbol * coefficient);
                        }

                        Some(acc)
                    })
                    .reduce(
                        || vec![Scalar::ZERO; symbol_count],
                        |mut a, b| {
                            // Element-wise addition of two partial results.
                            a.iter_mut().zip(b).for_each(|(x, y)| *x += y);
                            a
                        },
                    )
            }
        };

        #[cfg(not(feature = "parallel"))]
        let result = self.encode_inner(coding_vector);

        Ok(RLNCPacket { coding_vector: coding_vector.to_vec(), data: result })
    }

    /// Encodes the data with a random coding vector, using the provided random number generator.
    fn encode<R: Rng>(&self, mut rng: R) -> Result<RLNCPacket, RLNCError> {
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

        let encoder = SecureEncoder::new(data, chunk_count).unwrap();
        println!("{:?}", encoder);

        let packet = encoder.encode(rand::rng()).unwrap();

        println!("{:?}", packet);

        assert_eq!(packet.data.len(), encoder.chunk_size.div_ceil(SAFE_BYTES_PER_SCALAR));
    }
}
