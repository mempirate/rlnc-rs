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
    - Options: [Pedersen commitments](https://en.wikipedia.org/wiki/Pedersen_commitment), [LtHash](https://engineering.fb.com/2019/03/01/security/homomorphic-hashing/)

## Other Ideas
- [ ] If all peers in the network have known public keys, can we use that public key to determine the coding vector? What does that give us?
- [ ] LtHash for homomorphic hashing

Claude says:
 Major Benefits:

  - Bandwidth savings from implicit coding vectors
  - Fast GF(256) arithmetic with security guarantees
  - Network coordination capabilities
  - Attribution and accountability

  Suitable For:

  - Permissioned networks where public keys are known
  - Applications requiring accountability (financial, audit trails)
  - Bandwidth-constrained environments
  - Performance-critical scenarios where fast GF(256) is essential