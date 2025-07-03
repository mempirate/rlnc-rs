# `rslnc`

## Decoding

#### Standard Gaussian Elimination

In typical Gaussian elimination, you have a matrix equation $Ax = b$ and you:
1. Collect all equations upfront
2. Form the complete augmented matrix $[A|b]$
3. Perform forward elimination to get row echelon form
4. Perform back substitution to solve for $x$

#### RLNC Challenge: Online Processing

RLNC decoding has a key difference: packets arrive one by one over the network. You
can't wait to collect all equations before starting elimination!

Visual Walkthrough of RLNC Decoding

Let's trace through a 3-chunk example:

Initial State

Decoder state: empty
pivot_rows: [None, None, None]  // No pivots found yet
rank: 0

Packet 1 Arrives: [2, 3, 1] → data1

Step 1: eliminate_packet() - No existing pivots, so packet unchanged
Step 2: Find leading coefficient at column 0 (value 2)
Step 3: Normalize row by multiplying by 2⁻¹ in GF(256)
Step 4: Store as pivot for column 0

Matrix after Packet 1:
Row 0: [1, 1.5, 0.5] → normalized_data1
pivot_rows: [Some(0), None, None]
rank: 1

Packet 2 Arrives: [1, 2, 3] → data2

Step 1: eliminate_packet()
  - Use existing pivot at column 0 to eliminate coefficient 1
  - Subtract 1 * Row 0: [1, 2, 3] - 1*[1, 1.5, 0.5] = [0, 0.5, 2.5]

Step 2: Find leading coefficient at column 1 (value 0.5)
Step 3: Normalize by multiplying by (0.5)⁻¹ = 2
Step 4: Store as pivot for column 1

Matrix after Packet 2:
Row 0: [1, 1.5, 0.5] → normalized_data1
Row 1: [0, 1, 5] → normalized_data2
pivot_rows: [Some(0), Some(1), None]
rank: 2

Step 5: back_substitute() - Clean column 1 in previous rows
Row 0: [1, 1.5, 0.5] - 1.5*[0, 1, 5] = [1, 0, -7]

Packet 3 Arrives: [4, 5, 2] → data3

Step 1: eliminate_packet()
  - Use pivot at column 0: [4, 5, 2] - 4*[1, 0, -7] = [0, 5, 30]
  - Use pivot at column 1: [0, 5, 30] - 5*[0, 1, 5] = [0, 0, 5]

Step 2: Find leading coefficient at column 2 (value 5)
Step 3: Normalize by multiplying by 5⁻¹
Step 4: Store as pivot for column 2

Final Matrix:
Row 0: [1, 0, 0] → chunk0_data
Row 1: [0, 1, 0] → chunk1_data
Row 2: [0, 0, 1] → chunk2_data
pivot_rows: [Some(0), Some(1), Some(2)]
rank: 3 = chunk_count → DECODE COMPLETE!

Key Optimizations in the Implementation

1. Online Elimination (eliminate_packet)

```rs
fn eliminate_packet(&self, packet: &mut RLNCPacket) {
    // Process existing pivots in column order
    for (col, row) in self.pivot_rows.iter().enumerate()
        .filter_map(|(i, &r)| r.map(|r| (i, r))) {
        // Eliminate coefficient at this column
    }
}
```

Optimization: Eliminate against existing pivots immediately when packet arrives,
rather than waiting.

2. Efficient Pivot Tracking (pivot_rows)

```rs
pivot_rows: Vec<Option<usize>>  // pivot_rows[col] = Some(row_idx)
```

Optimization: Direct O(1) lookup to find which row contains the pivot for each column,
 instead of searching the matrix.

3. Incremental Back-Substitution (back_substitute)

```rs
fn back_substitute(&mut self, new_row_idx: usize) {
    // Only clean the NEW pivot column in PREVIOUS rows
    for i in 0..new_row_idx {
        // Eliminate new_pivot_col in row i
    }
}
```

Optimization: Instead of full back-substitution at the end, incrementally maintain
reduced form as pivots are added.

4. Early Termination

```rs
if self.rank >= self.chunk_count {
    return self.decode_final();  // Done! No need to process more packets
}
```
Optimization: Stop processing as soon as we have enough linearly independent packets.

5. Lazy Final Reconstruction (decode_final)

```rs
fn decode_final(&self) -> Result<Option<Bytes>, RLNCError> {
    // Matrix is already in reduced row echelon form!
    // Just extract the chunks and concatenate
}
```

Optimization: The matrix is maintained in solved form throughout, so final extraction
is just copying data.

Memory and Computational Benefits

| Aspect      | Basic Gaussian          | RLNC Decoder                 |
|-------------|-------------------------|------------------------------|
| Memory      | Store full n×n matrix   | Store only rank pivot rows   |
| Latency     | Wait for all packets    | Process packets immediately  |
| Computation | Full elimination at end | Incremental elimination      |
| Network     | Needs exactly n packets | Can handle redundant packets |

Visual Summary

```text
Traditional:    [Collect] → [Full Elimination] → [Back-Sub] → [Extract]
RLNC Decoder:   [Process packet] → [Immediate elimination] → [Incremental back-sub]
                      ↓                    ↓                        ↓
                   Always ready         Maintain RREF           Extract when rank = n
```

The RLNC decoder essentially maintains the matrix in Reduced Row Echelon Form (RREF)
throughout the process, rather than doing batch elimination at the end. This enables
immediate processing of packets as they arrive over the network!