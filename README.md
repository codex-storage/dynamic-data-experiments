Dynamic Data Experiments
================================
This is a prototype implementation of the proposed Codex storage proofs for dynamic data.

### Erasure Coding & Commitment
- [x] Organize data as byte Matrix with `k` rows and `m` columns
- [x] Convert the byte Matrix to Field Matrix with `k` rows and `m` columns
- [ ] Each cell in the Field matrix is "fat" (fat cell = `z` field elements) -> end up with `(k/z)`*`m` Matrix
- [x] Erasure code the columns -> end up with `n`*`m` Matrix
- [ ] Commit to each "fat" cell in each row independently with KZG
- [x] Commit to each row independently with KZG
- [ ] Build a Merkle tree with the KZG commitments 

**Note:** in the above I switched the directions of the encoding and commitment (opposite of the [proposal](https://hackmd.io/kPGC3VIZSaWj8DBYOjd4vA?view)) just because it was easier to implement but basically it is same thing.

### Sampling
- [ ] Select a set of columns randomly
- [ ] Generate a KZG evaluation proof at random point for each column
- [ ] Aggregate the KZG evaluation proofs

### Updating the Data
- [x] Select a row (or multiple)
- [x] Query the original row
- [x] Update the cells in that row
- [x] Erasure code the updated row

### Updating the Commitments
- [x] Query the old row and receive the new row 
- [ ] Compute the `delta` = `r'` - `r`
- [ ] Query the old the "fat" cell commitment and compute the new one
- [ ] Compute the `delta_comm` = `fat_comm'` - `fat_comm`
- [ ] Compute the new row commitment `row_comm'` = `row_comm` + `delta`

### Prove Data Update
- [ ] TODO...

### TODO:
- [ ] Clean up and optimize 
- [ ] Simulate interactions between Client (Data Owner) and SP (Storage Provider)
- [ ] Add details and write-up & experimentation/benchmark results 

**WARNING**: This repository contains work-in-progress prototypes, and has not received careful code review. It is NOT ready for production use.

