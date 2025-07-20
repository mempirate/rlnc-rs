//! Module that implements the RLNC decoding algorithm.

use crate::{
    common::RLNCError,
    matrix::Matrix,
    primitives::{ChunksError, field::Field, packet::RLNCPacket},
};

/// RLNC Decoder.
#[derive(Debug)]
pub struct Decoder<F: Field> {
    /// The size of each original chunk in bytes.
    chunk_size: usize,
    /// The number of coded packets required to decode the original data, also known as the
    /// generation size.
    chunk_count: usize,

    /// The RREF matrix of received coded packets.
    matrix: Matrix<F>,
}

impl<F: Field> Decoder<F> {
    /// Creates a new decoder for the given chunk size and chunk count (generation size).
    pub fn new(chunk_size: usize, chunk_count: usize) -> Result<Self, RLNCError> {
        if chunk_size == 0 {
            return Err(ChunksError::ZeroChunkSize.into());
        }

        if chunk_count == 0 {
            return Err(RLNCError::ZeroPacketCount);
        }

        Ok(Self { chunk_size, chunk_count, matrix: Matrix::new(chunk_count) })
    }

    /// Decodes a coded packet. If the decoder has enough linearly independent packets, it will
    /// return the original data.
    pub fn decode(&mut self, packet: RLNCPacket<F>) -> Result<Option<Vec<u8>>, RLNCError> {
        if packet.coding_vector.len() != self.chunk_count {
            return Err(RLNCError::InvalidCodingVectorLength(
                packet.coding_vector.len(),
                self.chunk_count,
            ));
        }

        if self.matrix.push_rref(packet) {
            return Ok(Some(self.matrix.decode(self.chunk_size)?));
        }

        // Store the packet data separately - we need coding vectors and data separate
        Ok(None)
    }

    /// Returns the number of linearly independent packets received.
    #[inline]
    pub const fn rank(&self) -> usize {
        self.matrix.rank()
    }

    /// Returns true if the decoder can decode the original data (i.e. if the rank is equal to the
    /// generation size).
    #[inline]
    pub const fn can_decode(&self) -> bool {
        self.matrix.can_decode()
    }
}
