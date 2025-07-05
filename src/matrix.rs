use crate::{common::RLNCError, primitives::packet::RLNCPacket};

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

        let mut chunks = vec![vec![0u8; chunk_size]; self.chunk_count];

        // Extract each chunk from the pivot rows (they're already normalized)
        for (col, row_idx) in self
            .pivots
            .iter()
            .enumerate()
            .filter_map(|(i, &r)| r.map(|r| (i, r)))
            .take(self.chunk_count)
        {
            let row = &self.data[row_idx];
            for i in 0..chunk_size {
                chunks[col][i] = row.data[i].into();
            }
        }

        // Reconstruct the original data by concatenating chunks
        let mut decoded = Vec::with_capacity(chunk_size * self.chunk_count);
        for chunk in chunks {
            decoded.extend_from_slice(&chunk);
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

            if !coeff.is_zero() {
                let pivot_row = &self.data[row];
                let pivot_coeff = pivot_row.coding_vector[col];

                let factor = (coeff / pivot_coeff).unwrap();
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
            if !coeff.is_zero() {
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
