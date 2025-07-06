# Roadmap

## 0.1.0: High-Performance RLNC (Non-Adversarial)
- [x] Basic RLNC coding and decoding with GF(256)
- [ ] Merkle tree authentication & integrity protection (source-only, no recoding)
- [ ] Support for custom binary extension fields (e.g. `GF(2^n)`), with some provided, optimized field implementations
- [ ] SIMD optimizations for `GF(2^8)`
- [ ] Network simulation

## 0.2.0: Cryptographically Secure RLNC (Adversarial)
- [ ] Prime field support
- [ ] Additive homomorphic authentication & integrity protection with Pedersen commitments
- [ ] Test [Bareiss algorithm](https://en.wikipedia.org/wiki/Bareiss_algorithm) for elimination
    - For large fields like `Scalar` from `curve25519-dalek`, division (which depends on inversion) is very slow. Bareiss algorithm is a way to reduce the number of inversions.
- [ ] Recoding with homomorphic authentication & integrity protection