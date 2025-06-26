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

impl Params{
    pub fn check_bounds(&self, r: usize, c: usize) -> anyhow::Result<()>{
        assert!(
            r < self.n,
            "row index {} out of bounds; must be < {}",
            r,
            self.n
        );
        assert!(
            c < self.m,
            "col index {} out of bounds; must be < {}",
            c,
            self.m
        );
        Ok(())
    }
    pub fn check_rows(&self, r: usize) -> anyhow::Result<()>{
        assert!(
            r < self.n,
            "row index {} out of bounds; must be < {}",
            r,
            self.n
        );
        Ok(())
    }

    pub fn check_cols(&self, c: usize) -> anyhow::Result<()>{
        assert!(
            c < self.m,
            "col index {} out of bounds; must be < {}",
            c,
            self.m
        );
        Ok(())
    }
}

/// data struct contains shards matrix with dimensions `n`*`m`
/// the matrix contains n rows, k of which are source data and the rest p = (n-k) are parity
#[derive(Clone, Debug)]
pub struct Data<T>{
    pub params: Params,
    pub matrix: Vec<Vec<T>>,
}

impl<T> Data<T>{

    pub fn get_row_mut(&mut self, idx: usize) -> &mut Vec<T>{
        &mut self.matrix[idx]
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

    fn get(&self, r: usize, c: usize) -> anyhow::Result<u8> {
        self.params.check_bounds(r,c)?;
        Ok(self.matrix[r][c].clone())
    }

    fn get_row(&self, r: usize) -> anyhow::Result<Vec<u8>> {
        self.params.check_rows(r)?;

        Ok(self.matrix[r].to_vec())
    }

    fn get_col(&self, c: usize) -> anyhow::Result<Vec<u8>> {
        self.params.check_cols(c)?;

        Ok(self.matrix
            .iter()
            .map(|row| row[c].clone())
            .collect())
    }

    fn set(&mut self, r: usize, c: usize, elem: u8) -> anyhow::Result<()> {
        self.params.check_bounds(r,c)?;
        self.matrix[r][c] = elem;
        Ok(())
    }

    /// Update col `c` in shards.
    /// given `new_col` will replace the column `c` or `shards[0..k][c]`
    fn update_col(&mut self, c: usize, new_col: &[u8]) -> anyhow::Result<()>{
        // sanity checks
        assert!(
            new_col.len() == self.params.k,
            "new_col length ({}) must equal k ({})",
            new_col.len(),
            self.params.k
        );
        self.params.check_cols(c)?;

        // write into each of the k data row at position c
        for i in 0..self.params.k {
            self.matrix[i][c] = new_col[i];
        }
        Ok(())
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

