use crate::{
    common::{RLNCError, SAFE_BYTES_PER_SCALAR},
    primitives::{
        field::{Field, Scalar},
        packet::RLNCPacket,
    },
};

/// A RREF matrix of coded packets, used to store the received coded packets and perform online
/// Gaussian elimination. To perform elimination efficiently, we store the pivots in a separate
/// array.
#[derive(Debug)]
pub(crate) struct Matrix {
    /// The number of original chunks (capacity of the matrix).
    chunk_count: usize,
    /// The received coded packets.
    data: Vec<RLNCPacket>,
    /// Maps pivot column index to row index. Array index is column index, value is row index.
    pivots: Vec<Option<usize>>,
    /// The number of linearly independent coded packets received (= rank of the matrix).
    rank: usize,
}

pub(crate) fn scalars_to_bytes(scalars: &[Scalar]) -> Vec<u8> {
    // Extract bytes from scalars - we stored 31 bytes per scalar
    scalars
        .iter()
        .flat_map(|scalar| {
            let bytes = scalar.to_bytes_le();
            // Return only the first 31 bytes (as we stored them)
            bytes[..SAFE_BYTES_PER_SCALAR].to_vec()
        })
        .collect()
}

impl Matrix {
    /// Creates a new matrix with the given chunk count.
    pub(crate) fn new(chunk_count: usize) -> Self {
        Self {
            chunk_count,
            data: Vec::with_capacity(chunk_count),
            pivots: vec![None; chunk_count],
            rank: 0,
        }
    }

    /// Decodes the original data from the matrix.
    pub(crate) fn decode(&self, chunk_size: usize) -> Result<Vec<u8>, RLNCError> {
        if !self.can_decode() {
            return Err(RLNCError::NotEnoughPackets(self.rank, self.chunk_count));
        }

        let scalars_per_chunk = chunk_size.div_ceil(SAFE_BYTES_PER_SCALAR);
        let mut chunk_scalars = vec![vec![Scalar::ZERO; scalars_per_chunk]; self.chunk_count];

        // Extract packed scalars from pivot rows (they're already normalized)
        for (col, row_idx) in self
            .pivots
            .iter()
            .enumerate()
            .filter_map(|(i, &r)| r.map(|r| (i, r)))
            .take(self.chunk_count)
        {
            let row = &self.data[row_idx];
            // Copy the packed scalars directly
            chunk_scalars[col].copy_from_slice(&row.data);
        }

        // Convert packed scalars back to bytes
        let mut decoded = Vec::with_capacity(chunk_size * self.chunk_count);
        for chunk in chunk_scalars {
            let chunk_bytes = scalars_to_bytes(&chunk);
            decoded.extend_from_slice(&chunk_bytes);
        }

        // Find the LAST boundary marker and truncate (since encoder places it at the end)
        let Some(boundary_pos) = decoded.iter().rposition(|&b| b == crate::common::BOUNDARY_MARKER)
        else {
            return Err(RLNCError::InvalidEncoding);
        };

        decoded.truncate(boundary_pos);
        Ok(decoded)
    }

    /// Pushes a new packet into the matrix, which will be eliminated against the existing rows.
    pub(crate) fn push_rref(&mut self, mut packet: RLNCPacket) -> bool {
        self.eliminate(&mut packet);

        if let Some(col) = packet.leading_coefficient() {
            if self.pivots[col].is_none() {
                // Normalize the packet so the leading coefficient is 1
                packet.normalize();

                // Store the pivot column -> row mapping
                self.pivots[col] = Some(self.data.len());
                self.data.push(packet);

                self.back_substitute(self.data.len() - 1);
                self.rank += 1;

                return self.can_decode();
            }
        }

        false
    }

    fn eliminate(&mut self, packet: &mut RLNCPacket) {
        // Process pivots in column order (array index order)
        for (col, row) in self
            .pivots
            .iter()
            .enumerate()
            .filter_map(|(i, &r)| r.map(|r| (i, r)))
            .take(self.chunk_count)
        {
            let coeff = packet.coding_vector[col];

            if !coeff.is_zero_vartime() {
                let pivot_row = &self.data[row];
                let pivot_coeff = pivot_row.coding_vector[col];

                let factor = coeff * pivot_coeff.invert().unwrap();
                packet.subtract_row(pivot_row, factor);
            }
        }
    }

    fn back_substitute(&mut self, new_row_idx: usize) {
        let new_row = &self.data[new_row_idx];
        let Some(new_pivot_col) = new_row.leading_coefficient() else {
            return;
        };

        let new_row = new_row.clone();

        for i in 0..new_row_idx {
            let coeff = self.data[i].coding_vector[new_pivot_col];
            if !coeff.is_zero_vartime() {
                let factor = coeff;
                self.data[i].subtract_row(&new_row, factor);
            }
        }
    }

    #[inline]
    pub(crate) const fn rank(&self) -> usize {
        self.rank
    }

    #[inline]
    pub(crate) const fn can_decode(&self) -> bool {
        self.rank >= self.chunk_count
    }
}
