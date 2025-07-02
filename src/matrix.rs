//! Matrix operations for the RLNC algorithm.
//!
//! The matrix is a 2D vector of GF256 values. The main operation defined in the module is
//! Gaussian elimination, which is used to solve the system of linear equations.
//!
//! ## Gaussian Elimination
//!
//! Gaussian elimination transforms a matrix into row echelon form to solve linear systems.
//! In RLNC decoding, we solve for original data packets from coded packet equations.
//!
//! ### Example in GF(2^8)
//!
//! Consider decoding 3 original packets X₁, X₂, X₃ from 3 coded packets:
//! ```text
//! Coded packet 1: Y₁ = 2⊗X₁ ⊕ 3⊗X₂ ⊕ 1⊗X₃ = [5, 7, 9]
//! Coded packet 2: Y₂ = 1⊗X₁ ⊕ 2⊗X₂ ⊕ 3⊗X₃ = [8, 4, 2]
//! Coded packet 3: Y₃ = 3⊗X₁ ⊕ 1⊗X₂ ⊕ 2⊗X₃ = [6, 1, 3]
//!
//! Matrix form: [C][X] = [Y]
//! [2 3 1] [X₁]   [5 7 9]
//! [1 2 3] [X₂] = [8 4 2]
//! [3 1 2] [X₃]   [6 1 3]
//!
//! Forward elimination creates upper triangular form:
//! Step 1: Make first column have leading 1
//! [1 ? ?] [X₁]   [? ? ?]
//! [0 ? ?] [X₂] = [? ? ?]
//! [0 ? ?] [X₃]   [? ? ?]
//!
//! Step 2: Eliminate second column below diagonal
//! [1 ? ?] [X₁]   [? ? ?]
//! [0 1 ?] [X₂] = [? ? ?]
//! [0 0 ?] [X₃]   [? ? ?]
//!
//! Step 3: Make diagonal element 1
//! [1 ? ?] [X₁]   [? ? ?]
//! [0 1 ?] [X₂] = [? ? ?]
//! [0 0 1] [X₃]   [? ? ?]
//!
//! Back substitution then recovers X₁, X₂, X₃ from bottom up.
//! ```
use crate::primitives::galois::GF256;

/// Eliminate the augmented matrix using Gaussian elimination.
pub fn eliminate(matrix: &mut [Vec<GF256>]) -> Vec<GF256> {
    let rows = matrix.len();
    let cols = matrix[0].len();

    // For an augmented matrix, we should have n equations for n unknowns + 1 augmentation column
    assert_eq!(rows, cols - 1);

    // Forward elimination - create upper triangular form
    for i in 0..rows {
        // Find pivot (non-zero element in column i)
        if matrix[i][i] == GF256::zero() {
            // Try to find a row below with non-zero element in column i and swap
            let mut found_pivot = false;
            for k in i + 1..rows {
                if matrix[k][i] != GF256::zero() {
                    matrix.swap(i, k);
                    found_pivot = true;
                    break;
                }
            }
            if !found_pivot {
                panic!("Matrix is singular - no pivot found for row {}", i);
            }
        }

        // Eliminate all elements below the pivot
        for j in i + 1..rows {
            if matrix[j][i] != GF256::zero() {
                let factor = (matrix[j][i] / matrix[i][i]).expect("Division possible");
                for k in i..cols {
                    let pivot_value = matrix[i][k];
                    matrix[j][k] -= factor * pivot_value;
                }
            }
        }
    }

    // Back substitution - eliminate above the diagonal
    for i in (1..rows).rev() {
        for j in 0..i {
            if matrix[j][i] != GF256::zero() {
                let factor = (matrix[j][i] / matrix[i][i]).expect("Division possible");
                for k in i..cols {
                    let pivot_value = matrix[i][k];
                    matrix[j][k] -= factor * pivot_value;
                }
            }
        }
    }

    // Extract solution vector (rightmost column divided by diagonal elements)
    let mut result = vec![GF256::zero(); rows];
    for i in 0..rows {
        if matrix[i][i] == GF256::zero() {
            panic!("Matrix is singular - zero diagonal element at position {}", i);
        }
        result[i] = (matrix[i][cols - 1] / matrix[i][i]).expect("Division possible");
    }

    result
}


pub fn rank(matrix: &[Vec<GF256>]) -> usize {
    matrix.iter().filter(|row| row.iter().any(|&elem| elem != GF256::zero())).count()
}
