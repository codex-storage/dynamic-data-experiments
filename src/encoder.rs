use std::marker::PhantomData;
use anyhow::{anyhow, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;
use crate::byte_data::{Data, Params};
use crate::traits::Encoder;

pub struct RSEncoder<T>{
    phantom_data: PhantomData<T>
}

impl RSEncoder<u8>{
    pub fn new() -> Self{
        Self{
            phantom_data: PhantomData::default()
        }
    }
}

impl Encoder<u8> for RSEncoder<u8> {
    type Params = Params;
    type DataMatrix<T> = Data<u8>;

    /// encode the columns of the data matrix in place
    fn encode(data: &mut Data<u8>) -> Result<()> {
        let n = data.params.n;
        assert!(data.params.k < n, "k must be less than total shards");
        let p = n - data.params.k;

        // ensure all rows are same length
        let row_size = data.matrix[0].len();
        for row in &data.matrix[1..] {
            assert_eq!(row.len(), row_size, "all rows must have equal length");
        }

        // build the encoder
        let rse = ReedSolomon::new(data.params.k, p)?;

        // prepare mutable slice references for in-place encode
        let mut shards_refs: Vec<&mut [u8]> = data.matrix.iter_mut()
            .map(|v| v.as_mut_slice())
            .collect();

        // encode
        rse.encode(&mut shards_refs)?;
        Ok(())
    }

    fn encode_col(data: &mut Data<u8>, c: usize) -> Result<Vec<u8>>{
        // bounds check
        if c >= data.params.m {
            return Err(anyhow!("col index {} out of bounds (< {})", c, data.params.m));
        }

        let n = data.params.n;
        let k = data.params.k;
        let p = n - k;

        // Build the column: data = existing byte, parity = zero
        let mut temp: Vec<Vec<u8>> = (0..n)
            .map(|i| {
                let byte = data.matrix[i][c];
                if i < k {
                    vec![byte]
                } else {
                    vec![0u8]
                }
            })
            .collect();
        let mut refs: Vec<&mut [u8]> = temp.iter_mut().map(|v| v.as_mut_slice()).collect();

        // Encode that stripe
        let rse = ReedSolomon::new(k, p)?;
        rse.encode(&mut refs)?;

        // Write back parity and collect full col
        let mut full_col = Vec::with_capacity(n);
        for i in 0..n {
            let b = refs[i][0];
            if i >= k {
                data.matrix[i][c] = b;
            }
            full_col.push(b);
        }

        Ok(full_col)
    }

    fn reconstruct(params: Params, matrix_opts: &mut Vec<Option<Vec<u8>>>) -> Result<()>{
        let n = params.n;
        let k = params.k;
        let p = n - k;
        let rse = ReedSolomon::new(k, p).unwrap();
        // reconstruct missing rows
        rse.reconstruct(matrix_opts)?;
        Ok(())
    }
}
