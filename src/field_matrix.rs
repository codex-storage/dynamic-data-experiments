use ark_ff::Field;
use ark_std::{test_rng};
use crate::byte_data::{Data, Params};
use crate::traits::DataMatrix;


/// a Field matrix with `row` number of rows and `cols` number of columns
#[derive(Clone, Debug)]
pub struct Matrix<F: Field + Clone> {
    pub params: Params,
    pub elms: Vec<Vec<F>>,
}

impl<F: Field + Clone> Matrix<F> {

    /// Creates a new matrix from given u8 data struct
    pub fn from_data(data: &Data<u8>) -> Self{
        let rows = data.params.n;
        let cols = data.params.m;

        let mut field_data = Vec::with_capacity(rows);
        for i in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for j in 0..cols {
                row.push(F::from(data.matrix[i][j]));
            }
            field_data.push(row);
        }
        Matrix { params: data.params.clone(), elms:field_data }
    }

    /// get mut the row at 0<idx<n
    pub fn get_row_mut(&mut self, idx: usize) -> anyhow::Result<&mut Vec<F>>{
        self.params.check_rows(idx)?;
        Ok(&mut self.elms[idx])
    }

    pub fn get_col_mut(&mut self, idx: usize) -> Vec<&mut F> {
        self.elms
            .iter_mut()
            .map(|row| &mut row[idx])
            .collect()
    }
}

impl<F: Field + Clone> DataMatrix<F> for Matrix<F> {
    type Params = Params;

    /// Generates a random matrix with given dimensions, uses given rng for randomness.
    fn new_random(params: Params) -> Self
    {
        let mut rng = test_rng();
        let rows = params.n;
        let cols = params.m;
        let mut data = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for _ in 0..cols {
                row.push(F::rand(&mut rng));
            }
            data.push(row);
        }
        Matrix { params, elms: data }
    }

    fn get(&self, r: usize, c: usize) -> anyhow::Result<F> {
        self.params.check_bounds(r,c)?;
        Ok(self.elms[r][c].clone())
    }

    fn set(&mut self, r: usize, c: usize, elem: F) -> anyhow::Result<()>{
        self.params.check_bounds(r,c)?;
        Ok(self.elms[r][c] = elem)
    }

    /// get the row at 0<idx<n
    fn get_row(&self, idx: usize) -> anyhow::Result<Vec<F>>{
        self.params.check_rows(idx)?;
        Ok(self.elms[idx].to_vec())
    }

    fn get_col(&self, idx: usize) -> anyhow::Result<Vec<F>> {
        self.params.check_cols(idx)?;
        Ok(self.elms
            .iter()
            .map(|row| row[idx].clone())
            .collect())
    }

    /// Print matrix
    fn pretty_print(&self) {
        for (i, shard) in self.elms.iter().enumerate() {
            print!("row {:>2}: ", i);
            for &b in shard {
                print!("{:>3} ", b);
            }
            println!();
        }
    }

    fn update_col(&mut self, c: usize, new_col: &[F]) -> anyhow::Result<()> {
        self.params.check_cols(c)?;

        // ensure the provided column has exactly `k` entries
        assert!(
            new_col.len() == self.params.k,
            "new_col length ({}) must equal k ({})",
            new_col.len(),
            self.params.k
        );

        for (r, val) in new_col.iter().enumerate() {
            self.elms[r][c] = val.clone();
        }

        Ok(())
    }
}


