use bytes::Bytes;

use super::galois::GF256;

#[derive(Debug, Clone)]
pub struct RLNCPacket {
    /// The coding vector (coefficients).
    pub coding_vector: Vec<GF256>,
    /// The actual data payload, containing a linear combination of the original data.
    pub data: Vec<GF256>,
}

impl RLNCPacket {
    /// Converts the coded packet into a row of the matrix.
    ///
    /// The row is a vector of GF256 values: `[coefficients | data]`.
    pub fn into_row(self) -> Vec<GF256> {
        let mut row = Vec::with_capacity(self.coding_vector.len() + self.data.len());
        row.extend(self.coding_vector);
        row.extend(self.data);
        row
    }
}
