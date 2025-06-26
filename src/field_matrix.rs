use ark_ff::Field;
use ark_std::rand::Rng;
use ark_std::UniformRand;
use crate::byte_data::{Data, Params};


/// a Field matrix with `row` number of rows and `cols` number of columns
#[derive(Clone, Debug)]
pub struct Matrix<F: Field + Clone> {
    pub params: Params,
    pub elms: Vec<Vec<F>>,
}

impl<F: Field + Clone> Matrix<F> {
    /// Creates a new matrix from given field data.
    pub fn new(params: Params, elms: Vec<Vec<F>>) -> Self {
        assert!(elms.len() == params.n,      "number of rows must match");
        for row in &elms {
            assert!(row.len() == params.m,   "each row must have `m` elements");
        }
        Matrix { params, elms }
    }

    /// Generates a random matrix with given dimensions, uses given rng for randomness.
    pub fn new_random<R: Rng + ?Sized>(params: Params, rng: &mut R) -> Self
        where
            F: UniformRand,
    {
        let rows = params.n;
        let cols = params.m;
        let mut data = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for _ in 0..cols {
                row.push(F::rand(rng));
            }
            data.push(row);
        }
        Matrix { params, elms: data }
    }

    /// Creates a new matrix from given data struct
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

    /// get the row at 0<idx<n
    pub fn row(&self, idx: usize) -> Vec<F>{
        assert!(idx < self.params.n, "Row index out of bounds");
        self.elms[idx].to_vec()
    }

    /// get mut the row at 0<idx<n
    pub fn row_mut(&mut self, idx: usize) -> &mut Vec<F>{
        assert!(idx < self.params.n, "Row index out of bounds");
        &mut self.elms[idx]
    }

    pub fn get_col(&self, idx: usize) -> Vec<&F> {
        self.elms
            .iter()
            .map(|row| &row[idx])
            .collect()
    }

    pub fn get_col_mut(&mut self, idx: usize) -> Vec<&mut F> {
        self.elms
            .iter_mut()
            .map(|row| &mut row[idx])
            .collect()
    }

    /// Print matrix
    pub fn pretty_print(&self) {
        for (i, shard) in self.elms.iter().enumerate() {
            print!("row {:>2}: ", i);
            for &b in shard {
                print!("{:>3} ", b);
            }
            println!();
        }
    }
}


