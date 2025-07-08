use thiserror::Error;

#[derive(Error, Debug)]
pub enum RLNCError {
    #[error("Data is empty")]
    EmptyData,
    #[error("Chunk count must be greater than 0")]
    ZeroChunkCount,
    #[error("Required packet count must be greater than 0")]
    ZeroPacketCount,
    #[error("Chunk size mismatch: got {0}, expected {1}")]
    ChunkSizeMismatch(usize, usize),
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

/// The number of bytes that can be safely stored in a BLS12-381 scalar without modular reduction.
///
/// BLS12-381 scalars are 255-bit numbers (32 bytes), but the field modulus is slightly less than
/// 2^255. By using only 31 bytes (248 bits), we guarantee the value is always less than the
/// modulus, avoiding the need for modular reduction and ensuring data integrity during
/// encode/decode.
pub(crate) const SAFE_BYTES_PER_SCALAR: usize = 31;
