use thiserror::Error;

#[derive(Error, Debug)]
pub enum RLNCError {
    #[error("Data is empty")]
    EmptyData,
    #[error("Chunk count must be greater than 0")]
    ZeroChunkCount,
    #[error("Required packet count must be greater than 0")]
    ZeroPacketCount,
    #[error("Coding vector length must match chunk count")]
    InvalidCodingVectorLength,
}

pub const BOUNDARY_MARKER: u8 = 0x81;
