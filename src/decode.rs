//! Module that implements the RLNC decoding algorithm.

use bytes::Bytes;

use crate::{
    common::RLNCError,
    primitives::{galois::GF256, packet::RLNCPacket},
};

#[derive(Debug, Clone)]
pub struct Decoder {
    /// The size of each original chunk in bytes.
    chunk_size: usize,
    /// The number of coded packets required to decode the original data.
    generation_size: usize,

    // Stateful data:
    /// The received coded packets.
    data: Vec<Vec<GF256>>,
    /// The number of linearly independent coded packets received (= rank of the matrix).
    rank: usize,
}

impl Decoder {
    pub fn new(chunk_size: usize, generation_size: usize) -> Result<Self, RLNCError> {
        if chunk_size == 0 {
            return Err(RLNCError::ZeroChunkCount);
        }

        if generation_size == 0 {
            return Err(RLNCError::ZeroPacketCount);
        }

        Ok(Self { chunk_size, generation_size, data: Vec::with_capacity(generation_size), rank: 0 })
    }

    /// Decodes a coded packet. If the decoder has enough linearly independent packets, it will
    /// return the original data.
    pub fn decode(&mut self, packet: RLNCPacket) -> Result<Option<Bytes>, RLNCError> {
        let rank_before = self.rank();

        if packet.coding_vector.len() != self.generation_size {
            return Err(RLNCError::InvalidCodingVectorLength);
        }

        self.data.push(packet.into_row());

        todo!("Finish decoding");

        Ok(None)
    }

    /// Returns the number of linearly independent packets received.
    fn rank(&self) -> usize {
        self.rank
    }
}
