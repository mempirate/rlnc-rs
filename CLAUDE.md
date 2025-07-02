# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Random Linear Network Coding (RLNC) library implemented in Rust, operating over the Galois Field GF(2^8). The library enables efficient data transmission in networks through encoding data into linear combinations that can be decoded from any sufficient set of coded packets.

## Development Commands

### Building and Testing
- `cargo build` - Build the project
- `cargo test` - Run all tests including property-based tests
- `cargo test test_encode_decode_with_random_vectors -- --nocapture` - Run the main integration test with output
- `cargo test --package rlnc --lib primitives::galois::tests` - Run specific Galois field tests
- `cargo run --example multicast` - Run the multicast example (currently empty placeholder)

### Code Quality
- `cargo clippy` - Run linting (extensive clippy configuration in Cargo.toml)
- `cargo fmt` - Format code using project's rustfmt.toml configuration
- `cargo doc` - Generate documentation

### Running Single Tests
- `cargo test <test_name>` - Run a specific test by name
- `cargo test --lib <module>::tests::<test_name>` - Run specific module test

## Architecture Overview

### Core Mathematical Foundation
The library is built around Galois Field GF(2^8) arithmetic, which provides the mathematical foundation for RLNC operations. All encoding/decoding operations are performed in this finite field.

### Key Components

**Galois Field Operations** (`src/primitives/galois.rs`)
- Complete GF(2^8) field implementation with precomputed lookup tables
- Supports all field operations: addition (XOR), multiplication, division, inverse
- Uses logarithm/exponentiation tables for efficient multiplication
- Includes comprehensive property-based tests verifying field axioms

**Packet Structure** (`src/primitives/packet.rs`)
- `RLNCPacket` contains coding vector (coefficients) and encoded data
- Both fields use `Vec<GF256>` for consistent field arithmetic
- Supports leading coefficient detection for Gaussian elimination

**Encoding Process** (`src/encode.rs`)
- Splits input data into fixed-size chunks with boundary marker at end of original data
- Generates coded packets as linear combinations: `Y = c₁⊗X₁ ⊕ c₂⊗X₂ ⊕ ... ⊕ cₖ⊗Xₖ`
- Supports both random and specified coding vectors
- Uses `BOUNDARY_MARKER` (0x81) to handle variable-length data and identify original data boundaries
- Contains TODO comments for SIMD and parallelization optimizations

**Decoding Process** (`src/decode.rs`)
- Complete implementation using Gaussian elimination with back-substitution
- Tracks pivot positions using `Vec<Option<usize>>` for column-to-row mapping
- Accumulates coded packets until sufficient rank is achieved
- Recovers original data through matrix inversion in GF(2^8)
- Handles boundary marker detection to return original data size

### Data Flow
1. **Encoding**: Original data → add boundary marker → chunks → linear combinations → coded packets
2. **Transmission**: Coded packets sent independently through network
3. **Decoding**: Collect coded packets → Gaussian elimination → solve linear system → extract chunks → find boundary marker → recover original data

### Implementation Status
- ✅ Complete: Galois field arithmetic, encoding, packet structure, decoding algorithm
- 🚧 Incomplete: Multicast example, SIMD optimizations
- 📝 Performance optimization opportunities identified in encode.rs

### Testing Strategy
The project uses proptest for property-based testing, particularly for verifying Galois field mathematical properties (associativity, commutativity, distributivity, etc.). The main integration test `test_encode_decode_with_random_vectors` validates the complete encode-decode cycle with 128KiB of random data.

### Error Handling
The library uses `thiserror` for structured error handling with the `RLNCError` enum covering:
- Empty data scenarios
- Zero chunk/packet counts  
- Coding vector length mismatches
- Invalid encoding detection

### Performance Considerations
- The encode loop in `encode_with_vector` is identified for SIMD optimization
- Galois field operations use lookup tables for efficiency
- Memory layouts designed for cache-friendly access patterns
- Decoding uses efficient pivot tracking to minimize matrix operations

### Development Notes
- Uses Rust stable 1.88.0 as specified in rust-toolchain.toml
- Extensive clippy configuration for code quality
- Custom rustfmt configuration for consistent formatting
- Property-based testing ensures mathematical correctness