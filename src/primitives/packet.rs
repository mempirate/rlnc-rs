use super::galois::GF256;

#[derive(Debug, Clone)]
pub struct RLNCPacket {
    /// The coding vector (coefficients).
    pub coding_vector: Vec<GF256>,
    /// The actual data payload, containing a linear combination of the original data.
    pub data: Vec<GF256>,
}

impl RLNCPacket {
    pub fn degree(&self) -> usize {
        self.coding_vector.iter().filter(|&c| !c.is_zero()).count()
    }

    pub fn leading_coefficient(&self) -> Option<(usize, GF256)> {
        self.coding_vector.iter().enumerate().find(|(_, c)| !c.is_zero()).map(|(i, c)| (i, *c))
    }
}
