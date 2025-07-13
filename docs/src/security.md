# Cryptographic Security

In RLNC, data is mutable, and can be combined and mutated in various ways. As long as these mutations are valid, receivers will be able to decode the original messages. But as soon as some invalid, bogus data is injected, receivers
may be unable to decode anything. These kinds of attacks are called [pollution attacks](https://en.wikipedia.org/wiki/Homomorphic_signatures_for_network_coding). Usually, message authentication
and integrity mechanisms would be deployed here, such as MACs or digital signatures over message digests.
But because data in RLNC is mutable, static message authentication mechanisms won't work. Any sort of message tag (MAC, signature, digest), should survive
valid linear combinations of the data.

## Additive Homomorphism
The property we're looking for is additive homomorphism.

## OLD
The general flow would work like this:
- Source node has known public key
- Source node divides data into chunks and commits to them (using signature)
- Source node encodes data into packets (i.e. combines chunks with coding vector)
- **The commitment is also combined, and remains valid!**
- Source node sends coded packets to the network
- Receiving nodes can individually ensure the authenticity of the coded packets, and the integrity of the data
- This specifically means that receiving nodes can verify that a) the original data was broadcast by the source node, and b) the data was not invalidly modified by a malicious node in the network

There are some options for authentication and integrity protection, all with different tradeoffs (and lack of implementations):

- [Merkle tree authentication](https://en.wikipedia.org/wiki/Merkle_tree)
    - This is the simplest option, and is the most widely used.
    - It's easy to implement, and is relatively efficient.
    - It only works in source-only mode, not with recoding.
    - Binding for small field sizes (e.g. `GF(2^8)`)
- [Pedersen commitments](https://en.wikipedia.org/wiki/Pedersen_commitment)
    - Proposed in https://ethresear.ch/t/faster-block-blob-propagation-in-ethereum/21370
    - Works in recoding setting because it's additive homomorphic
    - Requires large prime fields (e.g. `GF(2^256)`) to be binding (expensive in computation, at least 100x slower than `GF(2^8)`)
- [Homomorphic signatures](https://en.wikipedia.org/wiki/Homomorphic_signatures_for_network_coding)
    - Proposed in https://eprint.iacr.org/2006/025.pdf
    - Proposed in https://eprint.iacr.org/2011/018.pdf (lattice cryptography, secure even with small field sizes, PQ secure)
    - Works in recoding setting because it's additive homomorphic
    - **Have not found any production implementations!**
- [LtHash](https://engineering.fb.com/2019/03/01/security/homomorphic-hashing/)
    - Proposed in https://engineering.fb.com/2019/03/01/security/homomorphic-hashing/ (Note that this specifically won't work because it's only homomorphic for XOR operations)
    - Maybe there are variants of these hashes that are additively homomorphic or a way to leverage these for what we need?
    - **Unsure if this is secure with small fields, to investigate!**

It comes down to this: for authentication in full RLNC, you need either an additively homomorphic hash, commitment, or signature.
Options that are implemented in production (like Pedersen commitments) require large prime fields, which are expensive in computation.

Homomorphic signatures like in [this paper](https://eprint.iacr.org/2008/316.pdf) are not implemented in production and maybe not fully proven? But seem very promising.

**Sources:**
- [On Security Against Pollution Attacks in Network Coding Enabled 5G Networks](https://scispace.com/pdf/on-security-against-pollution-attacks-in-network-coding-33znx79vbi.pdf)
    - Provides a good overview of the problem and some solutions
- [Signing a Linear Subspace: Signature Schemes for Network Coding](https://eprint.iacr.org/2008/316.pdf)
    - Description of homomorphic signatures for network coding
- [Homomorphic Network Coding Signatures in the Standard Model](https://perso.uclouvain.be/benoit.libert/NCS-pkc11.pdf)
    - Improved version of the above paper
- [Efficient Network Coding Signatures in the Standard Model](https://www.iacr.org/archive/pkc2012/72930680/72930680.pdf)
    - More efficient implementation

- [On-the-Fly Verification of Rateless Erasure Codes for Efficient Content Distribution](https://pdos.csail.mit.edu/papers/otfvec/paper.pdf)
    - Describes a homomorphic hash function

- https://github.com/benwr/bromberg_sl2