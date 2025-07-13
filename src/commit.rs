//! This module implements non-hiding Pedersen commitments.
use blstrs::{G1Projective, Scalar};
use group::ff::Field;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// The domain separation tag for the Pedersen commitment scheme.
const DST: &[u8] = b"RLNC_PEDERSEN_GEN";

pub trait Committer {
    type Commitment;
    type Symbol: Field;

    /// Commits to the given symbols using the committer's generators.
    fn commit(&self, symbols: &[Self::Symbol]) -> Self::Commitment;
}

/// A committer that uses the non-hiding Pedersen commitment scheme.
///
/// # Idea
/// Pedersen commitments work with a number of generators, which are pre-computed and stored in the
/// committer. The commitment is computed as the sum of the generators multiplied by the symbols.
///
/// Since we don't want to transmit the generators to the verifier, we use deterministic
/// generators, which can be derived from a seed (like a sender's public key).
///
/// # Security
/// The security of the Pedersen commitment scheme relies on the discrete logarithm assumption.
/// The generators are chosen such that the discrete logarithm of the commitment to a symbol is
/// hard to compute.
#[derive(Debug)]
pub struct PedersenCommitter {
    generators: Vec<G1Projective>,
}

impl PedersenCommitter {
    /// Creates a new deterministic committer with the given seed and number of generators.
    pub fn new(seed: [u8; 32], n: usize) -> Self {
        #[cfg(feature = "parallel")]
        let generators = (0..n)
            .into_par_iter()
            .map(|i| {
                let mut msg = [0u8; 40];
                msg[..32].copy_from_slice(&seed);
                msg[32..].copy_from_slice(&i.to_le_bytes());

                G1Projective::hash_to_curve(&msg, DST, &[])
            })
            .collect();

        #[cfg(not(feature = "parallel"))]
        let generators = (0..n)
            .map(|i| {
                let mut msg = [0u8; 40];
                msg[..32].copy_from_slice(&seed);
                msg[32..].copy_from_slice(&i.to_le_bytes());

                G1Projective::hash_to_curve(&msg, DST, &[])
            })
            .collect();

        Self { generators }
    }
}

impl Committer for PedersenCommitter {
    type Commitment = G1Projective;
    type Symbol = Scalar;

    fn commit(&self, symbols: &[Scalar]) -> G1Projective {
        assert_eq!(symbols.len(), self.generators.len());

        G1Projective::multi_exp(&self.generators, symbols)
    }
}
