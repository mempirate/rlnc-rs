//! # RLNC - Random Linear Network Coding
//!
//! This library provides an implementation of Random Linear Network Coding (RLNC)
//! using Curve25519 scalar arithmetic.

mod common;
pub mod decode;
pub mod encode;
mod matrix;
mod primitives;

#[cfg(test)]
mod tests {
    use rand::Rng;
    use std::time::Instant;

    use super::{decode::Decoder, encode::Encoder};
    use curve25519_dalek::Scalar;

    #[test]
    fn test_encode_decode_with_random_vectors() {
        // 128KiB
        let data_size = 1024 * 128;
        let original_data = rand::rng().random_iter().take(data_size).collect::<Vec<_>>();

        let chunk_count = 10;

        println!(
            "Data size: {} bytes ({}KiB), chunk count: {}",
            data_size,
            data_size / 1024,
            chunk_count
        );

        let encoder = Encoder::new(original_data.clone(), chunk_count).unwrap();

        let mut coded_packets = Vec::with_capacity(chunk_count);

        let now = Instant::now();
        for _ in 0..chunk_count {
            let packet = encoder.encode(rand::rng()).unwrap();
            coded_packets.push(packet);
        }

        println!("Encoding time: {:?}", now.elapsed());

        let mut decoder = Decoder::new(encoder.chunk_size(), chunk_count).unwrap();

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
    fn test_single_byte_data() {
        let original_data = b"A";
        let chunk_count = 1;

        let encoder = Encoder::new(original_data, chunk_count).unwrap();
        let packet = encoder.encode_with_vector(&[Scalar::ONE]).unwrap();

        let mut decoder = Decoder::new(encoder.chunk_size(), chunk_count).unwrap();
        let decoded = decoder.decode(packet).unwrap();

        assert!(decoded.is_some());
        let decoded_data = decoded.unwrap();
        assert!(decoded_data.starts_with(original_data));
    }

    #[test]
    fn test_scalar_packing_unpacking() {
        use crate::encode::{scalars_to_bytes, bytes_to_scalars};
        
        let original_bytes = (0..62).collect::<Vec<u8>>(); // 62 bytes = 2 scalars (31 each)
        let scalars = bytes_to_scalars(&original_bytes);
        let unpacked_bytes = scalars_to_bytes(&scalars);
        
        println!("Original: {:?}", &original_bytes[..10]);
        println!("Unpacked: {:?}", &unpacked_bytes[..10]);
        println!("Original len: {}, Unpacked len: {}", original_bytes.len(), unpacked_bytes.len());
        
        // The unpacked should match the original exactly for this size
        assert_eq!(original_bytes, unpacked_bytes);
    }
}
