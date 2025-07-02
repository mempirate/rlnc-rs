//! Module that implements the RLNC decoding algorithm.

use bytes::{Bytes, BytesMut};

use crate::{
    common::RLNCError,
    primitives::{galois::GF256, packet::RLNCPacket},
};

/// Maximum supported generation size for static array allocation
const MAX_GENERATION_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub struct Decoder {
    /// The size of each original chunk in bytes.
    chunk_size: usize,
    /// The number of coded packets required to decode the original data.
    generation_size: usize,

    // Stateful data:
    /// The received coded packets.
    data: Vec<RLNCPacket>,
    /// Maps pivot column index to row index. Array index is column, value is row index.
    pivot_rows: [Option<usize>; MAX_GENERATION_SIZE],
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

        if generation_size > MAX_GENERATION_SIZE {
            return Err(RLNCError::InvalidCodingVectorLength);
        }

        Ok(Self {
            chunk_size,
            generation_size,
            data: Vec::with_capacity(generation_size),
            pivot_rows: [None; MAX_GENERATION_SIZE],
            rank: 0,
        })
    }

    /// Decodes a coded packet. If the decoder has enough linearly independent packets, it will
    /// return the original data.
    pub fn decode(&mut self, mut packet: RLNCPacket) -> Result<Option<Bytes>, RLNCError> {
        if packet.coding_vector.len() != self.generation_size {
            return Err(RLNCError::InvalidCodingVectorLength);
        }

        self.eliminate_packet(&mut packet);

        if let Some((col, _)) = packet.leading_coefficient() {
            if self.pivot_rows[col].is_none() {
                // Normalize the row so the leading coefficient is 1
                let leading_coeff = packet.coding_vector[col];
                if let Some(inv_coeff) = leading_coeff.inv() {
                    for i in 0..self.generation_size {
                        packet.coding_vector[i] = packet.coding_vector[i] * inv_coeff;
                    }

                    for i in 0..self.chunk_size {
                        packet.data[i] = packet.data[i] * inv_coeff;
                    }
                }

                self.pivot_rows[col] = Some(self.data.len());
                self.data.push(packet);
                self.rank += 1;

                self.back_substitute(self.data.len() - 1);
            }
        }

        if self.rank >= self.generation_size {
            return self.decode_final();
        }

        // Store the packet data separately - we need coding vectors and data separate
        Ok(None)
    }

    fn decode_final(&self) -> Result<Option<Bytes>, RLNCError> {
        let mut chunks = vec![vec![0u8; self.chunk_size]; self.generation_size];

        // Extract each chunk from the pivot rows (they're already normalized)
        for (col, row_idx) in self
            .pivot_rows
            .iter()
            .enumerate()
            .filter_map(|(i, &r)| r.map(|r| (i, r)))
            .take(self.generation_size)
        {
            let row = &self.data[row_idx];
            for i in 0..self.chunk_size {
                chunks[col][i] = row.data[i].into();
            }
        }

        // Reconstruct the original data by concatenating chunks
        let mut decoded = BytesMut::with_capacity(self.chunk_size * self.generation_size);
        for chunk in chunks {
            decoded.extend_from_slice(&chunk);
        }

        // Find the LAST boundary marker and truncate (since encoder places it at the end)
        let decoded_bytes = decoded.freeze();
        let Some(boundary_pos) =
            decoded_bytes.iter().rposition(|&b| b == crate::common::BOUNDARY_MARKER)
        else {
            return Err(RLNCError::InvalidEncoding);
        };

        Ok(Some(decoded_bytes.slice(0..boundary_pos)))
    }

    fn eliminate_packet(&self, packet: &mut RLNCPacket) {
        // Process pivots in column order (array index order)
        for (col, row) in self
            .pivot_rows
            .iter()
            .enumerate()
            .filter_map(|(i, &r)| r.map(|r| (i, r)))
            .take(self.generation_size)
        {
            let coeff = packet.coding_vector[col];

            if !coeff.is_zero() {
                let pivot_row = &self.data[row];
                let pivot_coeff = pivot_row.coding_vector[col];

                if let Some(factor) = coeff / pivot_coeff {
                    self.subtract_row(packet, pivot_row, factor);
                }
            }
        }
    }

    fn subtract_row(&self, dst: &mut RLNCPacket, src: &RLNCPacket, factor: GF256) {
        for i in 0..self.generation_size {
            dst.coding_vector[i] -= factor * src.coding_vector[i];
        }

        for i in 0..self.chunk_size {
            dst.data[i] -= factor * src.data[i];
        }
    }

    fn back_substitute(&mut self, new_row_idx: usize) {
        let new_row = &self.data[new_row_idx];
        let Some((new_pivot_col, _)) = new_row.leading_coefficient() else {
            return;
        };

        let new_row = new_row.clone();

        // Back-substitute against previous rows
        for i in 0..new_row_idx {
            let coeff = self.data[i].coding_vector[new_pivot_col];
            if !coeff.is_zero() {
                let factor = coeff;

                // Perform the subtraction operation manually to avoid borrowing conflicts
                for j in 0..self.generation_size {
                    self.data[i].coding_vector[j] -= factor * new_row.coding_vector[j];
                }

                for j in 0..self.chunk_size {
                    self.data[i].data[j] -= factor * new_row.data[j];
                }
            }
        }
    }

    /// Returns the number of linearly independent packets received.
    pub fn rank(&self) -> usize {
        self.rank
    }
}
