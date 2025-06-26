#[cfg(test)]
mod tests {
    use crate::byte_data::{Data, Params};
    use ark_poly::{EvaluationDomain};
    use reed_solomon_erasure::galois_8::ReedSolomon;
    use crate::kzg::{F, KZGPolyComm};
    use crate::field_matrix::Matrix;
    use ark_poly_commit::{Polynomial};
    use crate::encoder::RSEncoder;
    use crate::traits::{DataMatrix, Encoder, PolynomialCommitmentScheme};

    #[test]
    fn test_encode_columns() {
        // test parameters
        let k = 4;
        let p = 4;
        let n = k + p;
        let m = 8;

        // generate Data with random content
        let params = Params {
            k,
            n,
            m,
        };
        let mut data = Data::new_random(params);
        println!("data #row ={}", data.matrix.len());
        println!("data #col ={}", data.matrix[0].len());
        println!("data before encoding:");
        data.pretty_print();
        // original data matrix
        let original: Vec<Vec<u8>> = data.matrix[..k].to_vec();

        // encode
        RSEncoder::encode(&mut data).expect("encode failed");
        println!("data after encoding:");
        data.pretty_print();

        // verify data matrix unchanged
        assert_eq!(data.matrix[..k], original[..]);

        // simulate loss of one data and one parity rows
        let mut matrix_opts: Vec<_> = data.matrix.iter().cloned().map(Some).collect();
        matrix_opts[1] = None;
        matrix_opts[k] = None;

        // reconstruct missing rows
        RSEncoder::reconstruct(data.params.clone(), &mut matrix_opts).expect("reconstruction should succeed");

        // verify reconstruction for data shards
        for i in 0..k {
            let recovered = matrix_opts[i].clone().unwrap();
            assert_eq!(recovered, &original[i][..]);
        }
    }

    #[test]
    fn test_commit_rows() {
        // dimensions: 8 rows (4 parity), 8 columns
        let n = 8;
        let k = 4;
        let m = 8;

        // generate Data with random content
        let params = Params {
            k,
            n,
            m,
        };
        let mut data = Data::new_random(params.clone());
        RSEncoder::encode(&mut data).expect("encode failed");

        // make a random n×m matrix
        let matrix = Matrix::from_data(&data);

        // new kzg
        let kzg = KZGPolyComm::new(params);
        // setup kzg
        let srs = kzg.setup().expect("setup should succeed");

        // commit to its rows
        let kzg_comm = kzg.commit(&srs, &matrix).expect("commit_rows should succeed");


        let (row_polys, commitments, randomness) =
            kzg_comm.get_refs();

        // we produced exactly one polynomial, one comm, one rand per column
        assert_eq!(row_polys.len(), m);
        assert_eq!(commitments.len(), m);
        assert_eq!(randomness.len(), m);

        // check that each polynomial really interpolates its original rows
        for (i, poly) in row_polys.iter().enumerate() {
            let row = matrix.row(i);
            // evaluate poly at each domain point and collect
            let evals: Vec<_> = srs
                .ploycommit_domain
                .elements()
                .map(|x| poly.polynomial().evaluate(&x))
                .collect();
            assert_eq!(evals, row);
        }
    }

    #[test]
    fn test_open_commitments() {
        // dimensions: 8 rows (4 parity), 8 columns
        let n = 8;
        let k = 4;
        let m = 8;

        // generate Data with random content
        let params = Params {
            k,
            n,
            m,
        };
        let mut data = Data::new_random(params.clone());
        RSEncoder::encode(&mut data).expect("encode failed");

        // make a random n×m matrix
        let matrix = Matrix::from_data(&data);

        // new kzg
        let kzg = KZGPolyComm::new(params);
        // setup kzg
        let srs = kzg.setup().expect("setup should succeed");

        // commit to its rows
        let kzg_comm = kzg.commit(&srs, &matrix).expect("commit_rows should succeed");
        // check all cells
        for row in 0..n {
            for col in 0..m {
                let proof = KZGPolyComm::open(&kzg_comm, &srs, row, col)
                    .expect("open should succeed");
                let expected: F = matrix.row(row)[col].clone();

                assert!(
                    KZGPolyComm::verify(&kzg_comm, &srs, row, col, expected, &proof)
                        .expect("verify should succeed"),
                    "KZG open/verify failed for row={}, col={}",
                    row,
                    col
                );
            }
        }
    }

    #[test]
    fn test_update_col() {
        // dimensions: 8 rows (4 parity), 8 columns
        let n = 8;
        let k = 4;
        let m = 8;

        // generate Data with random content
        let params = Params {
            k,
            n,
            m,
        };
        // snapshot of original
        let mut data = Data::new_random(params);
        RSEncoder::encode(&mut data).expect("encode failed");
        println!("original data:");
        data.pretty_print();

        // pick a col and a new data‐col
        let c = 5;
        let new_col: Vec<u8> = (0..k)
            .map(|i| i as u8)
            .collect();

        // apply update
        data.update_col(c, &new_col);
        println!("data after update:");
        data.pretty_print();

        //data matrix [0..k) at col c must match new_col
        for i in 0..k {
            assert_eq!(
                data.matrix[i][c],
                new_col[i],
                "data matrix {} at row {} should be updated", i, c
            );
        }

        let _coded_row = RSEncoder::encode_col(&mut data, c).unwrap();
        println!("data after encoding update:");
        data.pretty_print();
    }

    #[test]
    fn test_update_commitments() -> anyhow::Result<()> {
        // dimensions: 8 rows (4 parity), 8 columns
        let n = 8;
        let k = 4;
        let m = 8;

        // generate Data with random content
        let params = Params {
            k,
            n,
            m,
        };
        // snapshot of original
        let mut data = Data::new_random(params.clone());
        RSEncoder::encode(&mut data).expect("encode failed");

        // Build a matrix where entry (i,j) = i * m + j
        let mut matrix = Matrix::<F>::from_data(&data);
        matrix.pretty_print();

        // new kzg
        let kzg = KZGPolyComm::new(params);
        // setup kzg
        let srs = kzg.setup().expect("setup should succeed");

        // commit to its rows
        let mut kzg_comm = kzg.commit(&srs, &matrix).expect("commit_rows should succeed");

        // a row to update
        let row_idx = 1;
        let old_row = matrix.row(row_idx);

        // a new row by adding a constant to each element
        let new_row: Vec<_> = old_row.iter()
            .map(|v| *v + F::from(10u64))
            .collect();

        // Apply the change to the in-memory matrix
        {
            let row_slice = matrix.row_mut(row_idx);
            for (j, val) in new_row.iter().enumerate() {
                row_slice[j] = *val;
            }
        }
        matrix.pretty_print();

        // do the comm update
        KZGPolyComm::update_commitments(&srs, &mut kzg_comm, row_idx, &old_row, &new_row)?;

        // Verify that each row polynomial now evaluates to the updated matrix
        for (i, poly) in kzg_comm.get_refs().0.iter().enumerate() {
            let evals: Vec<F> = srs.ploycommit_domain
                .elements()
                .map(|x| poly.polynomial().evaluate(&x))
                .collect();
            assert_eq!(evals, matrix.row(i));
        }

        // === new fresh commit on updated matrix ===
        let kzg_comm_fresh = kzg.commit(&srs, &matrix)?;
        // Compare each row commitment
        for (i, old_lbl_comm) in kzg_comm.get_refs().1.iter().enumerate() {
            let updated_comm = old_lbl_comm.commitment();
            let fresh_comm = kzg_comm_fresh.get_refs().1[i].commitment();
            assert_eq!(updated_comm, fresh_comm, "Row commitment mismatch at row {}", i);
        }


        Ok(())
    }
}
