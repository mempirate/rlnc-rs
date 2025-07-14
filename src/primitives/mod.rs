//! RLNC primitives.
pub mod field;
pub mod packet;
use field::Field;

/// A chunk of data.
#[derive(Debug, Clone)]
pub struct Chunk<F: Field> {
    symbols: Vec<F>,
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

    /// Returns the size of the chunk in bytes.
    pub(crate) fn size(&self) -> usize {
        self.size
    }
}

// TODO: Add a generic implementation for Chunk<S>. In its current form, we have to convert bytes to
// Symbols, but since ff::Field doesn't implement TryFrom<&[u8]>, we can't make it generic.
