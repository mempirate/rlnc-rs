//! Module that implements the RLNC decoding algorithm.

use bytes::Bytes;

use crate::{
    common::RLNCError,
    matrix,
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
        let _rank_before = self.rank();

        if packet.coding_vector.len() != self.generation_size {
            return Err(RLNCError::InvalidCodingVectorLength);
        }

        // Store the packet data separately - we need coding vectors and data separate
        self.data.push(packet.into_row());
        
        // Calculate rank based on the coding coefficient matrix only
        let coefficient_matrix: Vec<Vec<GF256>> = self.data.iter()
            .map(|row| row[..self.generation_size].to_vec())
            .collect();
        self.rank = matrix::rank(&coefficient_matrix);

        if self.rank() == self.generation_size {
            // We have enough linearly independent packets to decode
            // Solve the system for each byte position
            let mut chunk_solutions = vec![Vec::new(); self.generation_size];
            
            // For each byte position in the chunk
            for byte_pos in 0..self.chunk_size {
                // Build augmented matrix for this byte position: [coefficients | values]
                let mut byte_matrix: Vec<Vec<GF256>> = Vec::new();
                
                for packet_row in &self.data {
                    let mut row = Vec::new();
                    // Add coding coefficients
                    row.extend_from_slice(&packet_row[..self.generation_size]);
                    // Add the data value for this byte position
                    row.push(packet_row[self.generation_size + byte_pos]);
                    byte_matrix.push(row);
                }
                
                // Solve the linear system for this byte position
                let solution = matrix::eliminate(&mut byte_matrix);
                
                // Store each chunk's byte in the correct chunk
                for (chunk_idx, &chunk_byte) in solution.iter().enumerate() {
                    chunk_solutions[chunk_idx].push(u8::from(chunk_byte));
                }
            }

            // Reconstruct data by concatenating chunks
            let mut decoded_data = Vec::new();
            for chunk in chunk_solutions {
                decoded_data.extend(chunk);
            }

            return Ok(Some(Bytes::from(decoded_data)));
        }

        Ok(None)
    }

    /// Returns the number of linearly independent packets received.
    pub fn rank(&self) -> usize {
        matrix::rank(&self.data)
    }
}
