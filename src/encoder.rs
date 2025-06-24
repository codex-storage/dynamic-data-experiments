use anyhow::{anyhow, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;
use crate::byte_data::Data;
use crate::traits::Encoder;


impl Encoder<u8> for Data<u8> {
    /// encode the columns of the data matrix in place
    fn encode(&mut self) -> Result<()> {
        let n = self.params.n;
        assert!(self.params.k < n, "k must be less than total shards");
        let p = n - self.params.k;

        // ensure all shards are same length
        let shard_size = self.matrix[0].len();
        for shard in &self.matrix[1..] {
            assert_eq!(shard.len(), shard_size, "all shards must have equal length");
        }

        // build the encoder
        let rse = ReedSolomon::new(self.params.k, p)?;

        // prepare mutable slice references for in-place encode
        let mut shards_refs: Vec<&mut [u8]> = self.matrix.iter_mut()
            .map(|v| v.as_mut_slice())
            .collect();

        // encode
        rse.encode(&mut shards_refs)?;
        Ok(())
    }

    fn encode_col(&mut self, c: usize) -> Result<Vec<u8>>{
        // bounds check
        if c >= self.params.m {
            return Err(anyhow!("shard index {} out of bounds (< {})", c, self.params.m));
        }

        let n = self.params.n;
        let k = self.params.k;
        let p = n - k;

        // Build the column: data = existing byte, parity = zero
        let mut temp: Vec<Vec<u8>> = (0..n)
            .map(|i| {
                let byte = self.matrix[i][c];
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
                self.matrix[i][c] = b;
            }
            full_col.push(b);
        }

        Ok(full_col)
    }

    fn reconstruct(&mut self) -> Result<()>{
        todo!()
    }
}
