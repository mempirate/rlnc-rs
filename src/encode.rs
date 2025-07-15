//! Module that implements the RLNC encoding algorithm.
use rand::Rng;

use crate::{
    common::RLNCError,
    primitives::{Chunks, field::Field, packet::RLNCPacket},
};

/// RLNC encoder that's generic over the [`Field`] type. An ancoder should be instantiated
/// per piece of data the caller wants to encode, then used to generate the encoded chunks.
#[derive(Debug)]
pub struct Encoder<F: Field> {
    // The chunks of data to be encoded.
    chunks: Chunks<F>,
    // The number of chunks to split the data into (also known as the generation size).
    chunk_count: usize,
    // The size of each chunk in bytes.
    chunk_size: usize,
}

impl<F: Field> Encoder<F> {
    /// Creates a new encoder for the given data and chunk count.
    pub fn new(data: impl AsRef<[u8]>, chunk_count: usize) -> Result<Self, RLNCError> {
        let chunks = Self::prepare(data, chunk_count)?;
        let chunk_count = chunks.len();
        let chunk_size = chunks.chunk_size();

        Ok(Self { chunks, chunk_count, chunk_size })
    }

    /// Creates a new encoder from a vector of chunks.
    pub fn from_chunks(chunks: Chunks<F>) -> Self {
        let chunk_count = chunks.len();
        let chunk_size = chunks.chunk_size();

        Self { chunks, chunk_count, chunk_size }
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
    fn encode_inner(&self, coding_vector: &[F]) -> Vec<F> {
        let mut result = vec![F::ZERO; self.chunk_size.div_ceil(F::SAFE_CAPACITY)];

        for (chunk, &coefficient) in self.chunks.inner().iter().zip(coding_vector) {
            if coefficient.is_zero_vartime() {
                continue;
            }

            for (i, symbol) in chunk.symbols().iter().enumerate() {
                result[i] += *symbol * coefficient;
            }
        }

        result
    }

    /// Returns the number of chunks in the encoder.
    pub fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    /// Returns the size of each chunk in the encoder.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Prepares the data for encoding by splitting it into equally sized chunks and padding with
    /// zeros. Also converts the data into symbols in the chosen finite field.
    pub fn prepare(data: impl AsRef<[u8]>, chunk_count: usize) -> Result<Chunks<F>, RLNCError> {
        Ok(Chunks::new(data.as_ref(), chunk_count)?)
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
    pub fn encode_with_vector(&self, coding_vector: &[F]) -> Result<RLNCPacket<F>, RLNCError> {
        if coding_vector.len() != self.chunk_count {
            return Err(RLNCError::InvalidCodingVectorLength(coding_vector.len(), self.chunk_count));
        }

        let symbol_count = self.chunk_size.div_ceil(F::SAFE_CAPACITY);

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
                    .inner()
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
                        || vec![F::ZERO; symbol_count],
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
    pub fn encode<R: Rng>(&self, mut rng: R) -> Result<RLNCPacket<F>, RLNCError> {
        let coding_vector: Vec<F> = (0..self.chunk_count)
            .map(|_| {
                let mut bytes = [0u8; 32];
                rng.fill(&mut bytes[..F::SAFE_CAPACITY]);
                F::from_bytes(&bytes)
            })
            .collect();

        self.encode_with_vector(&coding_vector)
    }
}
