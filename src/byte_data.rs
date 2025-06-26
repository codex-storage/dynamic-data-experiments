use rand::Rng;
use crate::traits::DataMatrix;

/// parameters for the data
/// - k: number of data rows
/// - n: number of data + parity rows
/// - m: number of columns
#[derive(Clone, Debug)]
pub struct Params{
    pub k: usize,
    pub n: usize,
    pub m: usize,
}
/// data struct od type `T` contains data matrix with dimensions `n`*`m`
/// the matrix contains n rows, k of which are source data and the rest (n-k) are parity
#[derive(Clone, Debug)]
pub struct Data<T>{
    pub params: Params,
    pub matrix: Vec<Vec<T>>,
}

impl DataMatrix<u8> for Data<Option<u8>> {
    type Params = Params;

    /// new from random
    fn new_random(params: Self::Params) -> Self {
        let mut rng = rand::rng();
        // generate random data shards
        let matrix: Vec<Vec<Option<u8>>> = (0..params.n)
            .map(|i| {
                if i < params.k {
                    // data: random u8
                    (0..params.m).map(|_| Some(rng.random::<u8>())).collect()
                } else {
                    // parity: zero
                    vec![None; params.m]
                }
            })
            .collect();
        Self{
            params,
            matrix,
        }
    }

    /// Update col `c` in matrix.
    /// given `new_col` will replace the column `c` or `matrix[0..k][c]`
    fn update_col(&mut self, c: usize, new_col: &[u8]) {
        // sanity checks
        assert!(
            new_col.len() == self.params.k,
            "new_col length ({}) must equal k ({})",
            new_col.len(),
            self.params.k
        );
        assert!(
            c < self.params.m,
            "col index {} out of bounds; must be < {}",
            c,
            self.params.m
        );

        // write into each of the k data rows at position c
        for i in 0..self.params.n {
            if i < self.params.k {
                self.matrix[i][c] = Some(new_col[i]);
            }else{
                self.matrix[i][c] = None;
            }
        }
    }

    /// Print all matrix
    fn pretty_print(&self) {
        for (i, shard) in self.matrix.iter().enumerate() {
            print!("Row {:>2}: ", i);
            for opt in shard {
                match opt {
                    Some(byte) => print!("{:>3} ", byte),
                    None       => print!("  - "),
                }
            }
            println!();
        }
    }
}

