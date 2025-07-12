# Roadmap

## 0.1.0: High-Performance, Cryptographically-Secure RLNC
- [x] Basic RLNC coding and decoding with BLS12-381 scalar symbols
- [ ] Pedersen commitments for chunk authentication & integrity
- [ ] Encoding & decoding optimization
    - [x] Parallel encoding
- [ ] P2P network simulator
- [ ] Docs in mdbook

## 0.2.0: Extensibility
- [ ] Add regular RLNC over GF(256) with different encoder / decoder (SIMD)
- [ ] Support custom symbol sizes


## Other Ideas
- [ ] If all peers in the network have known public keys, can we use that public key to determine the coding vector? What does that give us?

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