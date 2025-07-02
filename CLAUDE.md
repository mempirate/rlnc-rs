# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Random Linear Network Coding (RLNC) library implemented in Rust, operating over the Galois Field GF(2^8). The library enables efficient data transmission in networks through encoding data into linear combinations that can be decoded from any sufficient set of coded packets.

## Development Commands

### Building and Testing
- `cargo build` - Build the project
- `cargo test` - Run all tests including property-based tests
- `cargo test --package rlnc --lib primitives::galois::tests` - Run specific Galois field tests
- `cargo run --example multicast` - Run the multicast example (currently empty)

### Code Quality
- `cargo clippy` - Run linting (extensive clippy configuration in Cargo.toml)
- `cargo fmt` - Format code using project's rustfmt.toml configuration
- `cargo doc` - Generate documentation

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
- Converts to matrix rows for linear algebra operations during decoding

**Encoding Process** (`src/encode.rs`)
- Splits input data into fixed-size chunks with boundary marker padding
- Generates coded packets as linear combinations: `Y = c₁⊗X₁ ⊕ c₂⊗X₂ ⊕ ... ⊕ cₖ⊗Xₖ`
- Supports both random and specified coding vectors
- Uses `BOUNDARY_MARKER` (0x81) to handle variable-length data

**Decoding Process** (`src/decode.rs`)
- Currently incomplete - contains skeleton for Gaussian elimination
- Designed to accumulate coded packets until sufficient rank is achieved
- Will recover original data through matrix inversion in GF(2^8)

**Matrix Operations** (`src/matrix.rs`)
- Placeholder for Gaussian elimination and matrix operations
- Critical for the decoding process

### Data Flow
1. **Encoding**: Original data → chunks → linear combinations → coded packets
2. **Transmission**: Coded packets sent independently through network
3. **Decoding**: Collect coded packets → build coefficient matrix → solve linear system → recover original data

### Implementation Status
- ✅ Complete: Galois field arithmetic, encoding, packet structure
- 🚧 Incomplete: Decoding algorithm, matrix operations, multicast example
- 📝 TODO comments indicate optimization opportunities (SIMD, parallelization)

### Testing Strategy
The project uses proptest for property-based testing, particularly for verifying Galois field mathematical properties (associativity, commutativity, distributivity, etc.).