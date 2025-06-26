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
/// data struct contains shards matrix with dimensions `n`*`m`
/// the matrix contains n rows, k of which are source data and the rest p = (n-k) are parity
#[derive(Clone, Debug)]
pub struct Data<T>{
    pub params: Params,
    pub matrix: Vec<Vec<T>>,
}

impl<T> Data<T>{
    pub fn get_row(&self, idx: usize) -> &Vec<T>{
        &self.matrix[idx]
    }

    pub fn get_row_mut(&mut self, idx: usize) -> &mut Vec<T>{
        &mut self.matrix[idx]
    }

    pub fn get_col(&self, idx: usize) -> Vec<&T> {
        self.matrix
            .iter()
            .map(|row| &row[idx])
            .collect()
    }

    pub fn get_col_mut(&mut self, idx: usize) -> Vec<&mut T> {
        self.matrix
            .iter_mut()
            .map(|row| &mut row[idx])
            .collect()
    }
}

impl DataMatrix<u8> for Data<u8> {
    type Params = Params;

    /// new from random
    fn new_random(params: Self::Params) -> Self {
        let mut rng = rand::rng();
        // generate random data shards
        let matrix: Vec<Vec<u8>> = (0..params.n)
            .map(|i| {
                if i < params.k {
                    // data: random u8
                    (0..params.m).map(|_| rng.random::<u8>()).collect()
                } else {
                    // parity: zero
                    vec![0u8; params.m]
                }
            })
            .collect();
        Self{
            params,
            matrix,
        }
    }

    /// Update col `c` in shards.
    /// given `new_col` will replace the column `c` or `shards[0..k][c]`
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

        // write into each of the k data row at position c
        for i in 0..self.params.k {
            self.matrix[i][c] = new_col[i];
        }
    }

    /// Print all shards
    fn pretty_print(&self) {
        for (i, shard) in self.matrix.iter().enumerate() {
            print!("Row {:>2}: ", i);
            for &b in shard {
                print!("{:>3} ", b);
            }
            println!();
        }
    }
}

