#[cfg(test)]
mod tests {
    use crate::byte_data::{Data, Params};
    use ark_poly::{EvaluationDomain};
    use crate::kzg10::{E, F, get_vk, KZG10PolyComm};
    use crate::field_matrix::Matrix;
    use ark_poly_commit::kzg10::Commitment;
    use crate::encoder::{BLSEncoder, BLSFieldEncoder, G8Encoder};
    use crate::matrix_commit::MatrixPolyComm;
    use crate::traits::{DataMatrix, Encoder, PolyCommScheme, MatrixPolyCommScheme, CommitOutputTrait, SRSTrait};

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
        G8Encoder::encode(&mut data).expect("encode failed");
        println!("data after encoding:");
        data.pretty_print();

        // verify data matrix unchanged
        assert_eq!(data.matrix[..k], original[..]);

        // simulate loss of one data and one parity rows
        let mut matrix_opts: Vec<_> = data.matrix.iter().cloned().map(Some).collect();
        matrix_opts[1] = None;
        matrix_opts[k] = None;

        // reconstruct missing rows
        G8Encoder::reconstruct(data.params.clone(), &mut matrix_opts).expect("reconstruction should succeed");

        // verify reconstruction for data shards
        for i in 0..k {
            let recovered = matrix_opts[i].clone().unwrap();
            assert_eq!(recovered, &original[i][..]);
        }
    }

    #[test]
    fn test_bls_encoder() {
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
        BLSEncoder::encode(&mut data).expect("encode failed");
        println!("data after encoding:");
        data.pretty_print();

        // verify data matrix unchanged
        assert_eq!(data.matrix[..k], original[..]);

        // simulate loss of one data and one parity rows
        let mut matrix_opts: Vec<_> = data.matrix.iter().cloned().map(Some).collect();
        matrix_opts[1] = None;
        matrix_opts[k] = None;

        // TODO: reconstruct missing rows
        // BLSEncoder::reconstruct(data.params.clone(), &mut matrix_opts).expect("reconstruction should succeed");

        // verify reconstruction for data shards
        // for i in 0..k {
        //     let recovered = matrix_opts[i].clone().unwrap();
        //     assert_eq!(recovered, &original[i][..]);
        // }
    }

    #[test]
    fn test_bls_field_encoder() {
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
        let data = Data::new_random(params);
        println!("data #row ={}", data.matrix.len());
        println!("data #col ={}", data.matrix[0].len());
        println!("data before encoding:");
        data.pretty_print();
        // original data matrix
        let mut original = Matrix::from_data(&data);
        let original_copy = Matrix::from_data(&data);
        println!("data as Field elements:");
        original.pretty_print();

        // encode
        BLSFieldEncoder::encode(&mut original).expect("encode failed");
        println!("data after encoding:");
        original.pretty_print();

        // verify data matrix unchanged
        assert_eq!(original.elms[..k], original_copy.elms[..k]);

        // simulate loss of one data and one parity rows
        let mut matrix_opts: Vec<_> = data.matrix.iter().cloned().map(Some).collect();
        matrix_opts[1] = None;
        matrix_opts[k] = None;

        // TODO: reconstruct missing rows
        // BLSEncoder::reconstruct(data.params.clone(), &mut matrix_opts).expect("reconstruction should succeed");

        // verify reconstruction for data shards
        // for i in 0..k {
        //     let recovered = matrix_opts[i].clone().unwrap();
        //     assert_eq!(recovered, &original[i][..]);
        // }
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
        G8Encoder::encode(&mut data).expect("encode failed");

        // make a random n×m matrix
        let matrix = Matrix::from_data(&data);

        // degree is the size of each row (the number of cells in a row) which equals the number of columns
        let degree = m;
        // setup kzg
        type P = KZG10PolyComm;
        type C = MatrixPolyComm<F,P>;
        let srs = C::setup(degree).expect("setup should succeed");

        // commit to its rows
        let kzg_comm = C::commit(&srs, &matrix).expect("commit_rows should succeed");

        assert_eq!(kzg_comm.comm_output.len(), m);

        // check that each polynomial is really the original rows
        for i in 0..m {
            let row = matrix.get_row(i).unwrap();
            let evals: Vec<_> = kzg_comm.get_poly(i).coeffs.clone();
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
        G8Encoder::encode(&mut data).expect("encode failed");

        // make a random n×m matrix
        let matrix = Matrix::from_data(&data);

        // degree is the size of each row (the number of cells in a row) which equals the number of columns
        let degree = m;
        // setup kzg
        type P = KZG10PolyComm;
        type C = MatrixPolyComm<F,P>;
        let srs = C::setup(degree).expect("setup should succeed");

        // commit to its rows
        let kzg_comm = C::commit(&srs, &matrix).expect("commit_rows should succeed");

        // verifier Part
        let vk = get_vk(&srs.pp).unwrap();
        let verifier_comms: Vec<Commitment<E>> = kzg_comm.comm_output.iter().map(|c|c.get_comm().clone()).collect();

        // check all domain points
        for row in 0..n {
            for col in 0..m {
                let proof = C::open(&kzg_comm, &srs, row, col)
                    .expect("open should succeed");
                let expected: F = matrix.elms[row][col].clone();
                let point = srs.get_domain_element(col);
                assert!(
                    C::verify(&vk, &verifier_comms[row], point, expected, &proof)
                        .expect("verify should succeed"),
                    "KZG open/verify failed for row={}, col={}",
                    row,
                    col
                );
            }
        }
    }

    #[test]
    fn test_update_col(){
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
        G8Encoder::encode(&mut data).expect("encode failed");
        println!("original data:");
        data.pretty_print();

        // pick a col and a new data‐col
        let c = 5;
        let new_col: Vec<u8> = (0..k)
            .map(|i| i as u8)
            .collect();

        // apply update
        data.update_col(c, &new_col).expect("update col");
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

        G8Encoder::encode_col(&mut data, c).expect("encode col");
        println!("data after encoding update:");
        data.pretty_print();
    }

    #[test]
    fn test_kzg10_update_commitments() {
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
        // original
        let mut data = Data::new_random(params.clone());
        G8Encoder::encode(&mut data).expect("encode failed");

        // Build a matrix where entry (i,j) = i * m + j
        let matrix = Matrix::<F>::from_data(&data);

        // degree is the size of each row (the number of cells in a row) which equals the number of columns
        let degree = m;
        // setup kzg
        type P = KZG10PolyComm;
        let srs = P::setup(degree).expect("setup should succeed");
        let mut row = matrix.get_row(0).expect("get row");
        let mut com = P::commit(&srs, row.clone()).expect("commit");

        // Verify that row polynomial coeffs are the row data
        for i in 0..m {
            let row_elem = row[i].clone();
            let eval = com.poly.coeffs[i].clone();
            assert_eq!(eval, row_elem);
        }

        let cell = row[0].clone();
        let new_cell = cell + F::from(10u64);

        P::update_commitment(&srs,&mut com,cell,new_cell.clone(),0).expect("update comm");

        let eval = com.poly.coeffs[0].clone();
        assert_eq!(eval, new_cell);

        row[0] = new_cell;

        let new_com = P::commit(&srs, row.clone()).expect("commit");

        assert_eq!(com.comm, new_com.comm);

    }

    #[test]
    fn test_update_commitments() {
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
        G8Encoder::encode(&mut data).expect("encode failed");

        // Build a matrix where entry (i,j) = i * m + j
        let mut matrix = Matrix::<F>::from_data(&data);
        println!("---------- original ------------");
        matrix.pretty_print();

        // degree is the size of each row (the number of cells in a row) which equals the number of columns
        let degree = m;
        // setup kzg
        type P = KZG10PolyComm;
        type C = MatrixPolyComm<F,P>;
        let srs = C::setup(degree).expect("setup should succeed");

        // commit to its rows
        let mut kzg_comm = C::commit(&srs, &matrix).expect("commit_rows should succeed");

        // a column to update
        let col_idx = 1;
        let old_col = matrix.get_col(col_idx).expect("get old col");

        let new_col_data: Vec<_> = old_col
            .iter()
            .take(k)                              // only look at the first k entries
            .map(|v| *v + F::from(10u64))        // then do your +10
            .collect();

        matrix.update_col(col_idx, &new_col_data).expect("update col");
        println!("---------- updated ------------");
        matrix.pretty_print();
        // G8Encoder::encode_col(&mut data, col_idx).expect("encode col");
        // println!("---------- encoded ------------");
        // matrix.pretty_print();

        let encoded_new_col = matrix.get_col(col_idx).expect("get old col");

        // do the comm update
        C::update_commitments(&srs, &mut kzg_comm, col_idx, &old_col, &encoded_new_col).expect("update comm");

        // Verify that each row polynomial now evaluates to the updated matrix
        for i in 0..m {
            let row = matrix.get_row(i).unwrap();
            // evaluate poly at each domain point and collect
            let evals: Vec<_> = kzg_comm.get_poly(i).coeffs.clone();
            assert_eq!(evals, row);
        }

        // === new fresh commit on updated matrix ===
        let kzg_comm_fresh = C::commit(&srs, &matrix).expect("commit updated matrix");
        // Compare each row commitment
        for (i, old_lbl_comm) in kzg_comm.comm_output.iter().enumerate() {
            let updated_comm = old_lbl_comm.get_comm();
            let fresh_comm = kzg_comm_fresh.get_comm(i);
            assert_eq!(updated_comm, fresh_comm, "Row commitment mismatch at row {}", i);
        }

    }
}
