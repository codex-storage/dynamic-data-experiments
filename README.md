Dynamic Data Experiments
================================
This is a prototype implementation of the proposed Codex storage proofs for dynamic data.

### Erasure Coding & Commitment
- [x] Organize data as byte Matrix with `k` rows and `m` columns
- [x] Convert the byte Matrix to Field Matrix with `k` rows and `m` columns
- [x] Erasure code the columns -> end up with `n`*`m` Matrix
- [x] Commit to each row independently with KZG

**Note:** in the above I switched the directions of the encoding and commitment (opposite of the [proposal](https://hackmd.io/kPGC3VIZSaWj8DBYOjd4vA?view)) just because it was easier to implement but basically it is same thing.

### Sampling
- [x] Select a set of rows randomly
- [x] Generate a KZG evaluation proof at random point for each selected row

### Updating the Data
- [x] Select a column (or multiple)
- [x] Query the original column
- [x] Update the cells in that column
- [x] Erasure code the updated column

### Updating the Commitments
- [x] Query the old column and receive the new column
- [x] Iterate over all row commitments 
- [x] Commit to each old cell `c_i` and new cell `c'_i` in each row `i`:
- [x] Compute `delta_i` = `c'_i` - `c_i`
- [x] Compute the new row commitment `row_comm_i'` = `row_comm_i` + `delta_i`

### Prove Data & Commitment Update
- [ ] TODO...

### Additional functionalities
- [x] BLS encoder: erasure coding over Bls12_381

### TODO:
- [ ] implement matrix with "fat" cell and let encoding and commitment work over such matrix.
- [ ] fix conversion between byte to field matrix.
- [ ] Aggregate the KZG proofs.
- [ ] Build a Merkle tree with the KZG commitments.
- [ ] Simulate interactions between Client (Data Owner) and SP (Storage Provider).
- [ ] Clean up and optimize.
- [ ] Add details and write-up & experimentation/benchmark results.

**WARNING**: This repository contains work-in-progress prototypes, and has not received careful code review. It is NOT ready for production use.

