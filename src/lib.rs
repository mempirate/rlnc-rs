//! # RLNC - Random Linear Network Coding
//!
//! This library provides an implementation of Random Linear Network Coding (RLNC)
//! using BLS12-381 scalar arithmetic.

pub mod commit;
mod common;
pub mod decode;
pub mod encode;
mod matrix;
pub mod primitives;

#[cfg(test)]
mod tests {
    use blstrs::G1Projective;
    use group::ff::Field;
    use rand::Rng;
    use std::time::{Duration, Instant};

    use crate::commit::PedersenCommitter;

    use super::{decode::Decoder, encode::Encoder, primitives::field::Scalar};

    #[test]
    fn test_encode_decode_with_random_vectors() {
        // 128KiB
        let data_size = 1024 * 1024;
        let original_data = rand::rng().random_iter().take(data_size).collect::<Vec<_>>();

        let chunk_count = 5;

        println!("Data size: {}KiB, chunk count: {}", data_size / 1024, chunk_count);

        let encoder = Encoder::<Scalar>::new(original_data.clone(), chunk_count).unwrap();
        println!("Chunk size: {}", encoder.chunk_size());

        let mut coded_packets = Vec::with_capacity(chunk_count);

        let now = Instant::now();
        for _ in 0..chunk_count {
            let packet = encoder.encode(rand::rng()).unwrap();
            coded_packets.push(packet);
        }

        println!("Encoding time: {:?}", now.elapsed());

        let mut decoder = Decoder::<Scalar>::new(encoder.chunk_size(), chunk_count).unwrap();

        let now = Instant::now();
        let decoded = loop {
            if let Some(decoded) = decoder.decode(coded_packets.pop().unwrap()).unwrap() {
                break decoded;
            }
        };

        println!("Decoding time: {:?}", now.elapsed());

        println!("Decoded length: {}", decoded.len());
        println!("Original length: {}", original_data.len());
        assert!(decoded.starts_with(&original_data));
    }

    #[test]
    fn test_simple_commit_and_verify() {
        let seed = [0u8; 32];

        let data = rand::rng().random_iter().take(1024 * 512).collect::<Vec<_>>();
        let chunk_count = 10;

        let chunks = Encoder::<Scalar>::prepare(&data, chunk_count).unwrap();
        let start = Instant::now();
        let committer = PedersenCommitter::new(seed, chunks.inner()[0].symbols().len());
        println!("Committer creation time: {:?}", start.elapsed());

        let start = Instant::now();
        let commitments =
            chunks.inner().iter().map(|c| committer.commit(c.symbols())).collect::<Vec<_>>();
        println!("Commitment time: {:?}", start.elapsed());

        let encoder = Encoder::from_chunks(chunks);
        let mut decoder = Decoder::<Scalar>::new(encoder.chunk_size(), chunk_count).unwrap();

        let mut coded_packets =
            (0..chunk_count).map(|_| encoder.encode(rand::rng()).unwrap()).collect::<Vec<_>>();

        let mut verify_time = Duration::ZERO;

        let decoded = loop {
            let next = coded_packets.pop().unwrap();
            // Verify:
            // 1. MSM commitments with coefficients
            // 2. Recompute commitment over encoded data
            // 3. Compare
            let start = Instant::now();
            let msm = G1Projective::multi_exp(&commitments, &next.coding_vector);
            let com = committer.commit(&next.data);
            verify_time += start.elapsed();

            assert_eq!(msm, com);
            if let Some(decoded) = decoder.decode(next).unwrap() {
                break decoded;
            }
        };

        println!("Total verification time: {verify_time:?}");

        assert!(decoded.starts_with(&data));
    }

    #[test]
    fn test_single_byte_data() {
        let original_data = b"A";
        let chunk_count = 1;

        let encoder = Encoder::<Scalar>::new(original_data, chunk_count).unwrap();
        let packet = encoder.encode_with_vector(&[Scalar::ONE]).unwrap();

        let mut decoder = Decoder::<Scalar>::new(encoder.chunk_size(), chunk_count).unwrap();
        let decoded = decoder.decode(packet).unwrap();

        assert!(decoded.is_some());
        let decoded_data = decoded.unwrap();
        assert!(decoded_data.starts_with(original_data));
    }
}
