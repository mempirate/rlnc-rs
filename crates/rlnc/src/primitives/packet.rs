//! RLNC coded packet.
use super::field::Field;

/// A coded packet.
#[derive(Debug, Clone)]
pub struct RLNCPacket<F: Field> {
    /// The coding vector (coefficients).
    pub coding_vector: Vec<F>,
    /// The actual data payload, containing a linear combination of the original data.
    pub data: Vec<F>,
}

impl<F: Field> RLNCPacket<F> {
    /// Returns the number of non-zero coefficients in the coding vector.
    pub fn degree(&self) -> usize {
        self.coding_vector.iter().filter(|&c| !c.is_zero_vartime()).count()
    }

    /// Returns the index of the leading coefficient (non-zero coefficient).
    pub fn leading_coefficient(&self) -> Option<usize> {
        self.coding_vector.iter().position(|c| !c.is_zero_vartime())
    }

    /// Normalizes the packet so the leading coefficient is 1.
    pub fn normalize(&mut self) {
        if let Some(col) = self.leading_coefficient() {
            let inv = self.coding_vector[col].invert().unwrap();

            for i in 0..self.coding_vector.len() {
                self.coding_vector[i] = self.coding_vector[i] * inv;
            }

            for i in 0..self.data.len() {
                self.data[i] = self.data[i] * inv;
            }
        }
    }

    /// Subtracts the `src` row from the current row in place, multiplying by `factor`.
    pub fn subtract_row(&mut self, src: &Self, factor: F) {
        for i in 0..self.coding_vector.len() {
            self.coding_vector[i] -= factor * src.coding_vector[i];
        }

        for i in 0..self.data.len() {
            self.data[i] -= factor * src.data[i];
        }
    }
}
