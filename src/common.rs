use thiserror::Error;

#[derive(Error, Debug)]
pub enum RLNCError {
    #[error("Data is empty")]
    EmptyData,
    #[error("Chunk count must be greater than 0")]
    ZeroChunkCount,
    #[error("Required packet count must be greater than 0")]
    ZeroPacketCount,
    #[error("Coding vector length must match chunk count: got {0}, expected {1}")]
    InvalidCodingVectorLength(usize, usize),
    #[error("Invalid encoding")]
    InvalidEncoding,
    #[error("Not enough linearly independent packets to decode, have {0}, need {1}")]
    NotEnoughPackets(usize, usize),
}

/// The boundary marker is a special byte that is used to separate the encoded data from the
/// padding.
pub(crate) const BOUNDARY_MARKER: u8 = 0x81;
