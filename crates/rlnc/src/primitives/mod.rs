//! RLNC primitives.
pub mod field;
pub mod packet;
use field::Field;

use crate::common::BOUNDARY_MARKER;

/// A collection of equally sized, prepared chunks of data. Each chunk of data holds the symbols.
/// This type represents correctly sized and padded chunks of data that are ready to be encoded.
#[derive(Debug)]
pub struct Chunks<F: Field> {
    inner: Vec<Chunk<F>>,
    chunk_size: usize,
}

/// Errors that can occur when creating a new collection of chunks.
#[derive(Debug, thiserror::Error)]
pub enum ChunksError {
    /// The data is empty.
    #[error("data is empty")]
    EmptyData,
    /// The chunk count is zero.
    #[error("chunk count is zero")]
    ZeroChunkCount,
    /// The chunk size is zero.
    #[error("chunk size is zero")]
    ZeroChunkSize,
}

impl<F: Field> Chunks<F> {
    /// Creates a new collection of chunks from a slice of bytes. The data is split into
    /// `chunk_count` equally sized chunks, and then converted into symbols (scalars) of the
    /// field `F`. See also [`Chunk`] for more details.
    pub fn new(data: &[u8], chunk_count: usize) -> Result<Self, ChunksError> {
        if data.is_empty() {
            return Err(ChunksError::EmptyData);
        }

        if chunk_count == 0 {
            return Err(ChunksError::ZeroChunkCount);
        }

        let mut data = Vec::from(data.as_ref());
        data.push(BOUNDARY_MARKER);

        // Calculate chunk size to accommodate original data + boundary marker
        let chunk_size = data.len().div_ceil(chunk_count);

        // Round up chunk size to nearest multiple of `F::SAFE_CAPACITY` for scalar packing
        let chunk_size = chunk_size.div_ceil(F::SAFE_CAPACITY) * F::SAFE_CAPACITY;
        let padded_len = chunk_size * chunk_count;

        // Pad the rest with zeros if needed
        data.resize(padded_len, 0);

        let chunks = data.chunks_exact(chunk_size).map(Chunk::from_bytes).collect();

        Ok(Self { inner: chunks, chunk_size })
    }

    /// Returns the size of the chunks in bytes.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Returns the inner chunks.
    pub fn inner(&self) -> &[Chunk<F>] {
        &self.inner
    }

    /// Returns the number of chunks in the collection.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// A chunk of data.
#[derive(Debug, Clone)]
pub struct Chunk<F: Field> {
    symbols: Vec<F>,
    #[allow(unused)]
    size: usize,
}

impl<F: Field> Chunk<F> {
    /// Creates a new chunk from a slice of bytes, and converts it into a vector of scalars
    /// (symbols used for encoding).
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let size = bytes.len();
        Self { symbols: bytes.chunks(F::SAFE_CAPACITY).map(|c| F::from_bytes(c)).collect(), size }
    }

    /// Returns the symbols of the chunk.
    pub(crate) fn symbols(&self) -> &[F] {
        &self.symbols
    }
}
