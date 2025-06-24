use ark_ff::Field;
use ark_std::rand::Rng;
use ark_std::UniformRand;
use crate::byte_data::Data;


/// a Field matrix with `row` number of rows and `cols` number of columns
#[derive(Clone, Debug, PartialEq)]
pub struct Matrix<F: Field + Clone> {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Vec<F>>,
}

impl<F: Field + Clone> Matrix<F> {
    /// Creates a new matrix from given field data.
    pub fn new(rows: usize, cols: usize, data: Vec<Vec<F>>) -> Self {
        assert!(data.len() == rows,      "number of rows must match");
        for row in &data {
            assert!(row.len() == cols,   "each row must have `cols` elements");
        }
        Matrix { rows, cols, data }
    }

    /// Generates a random matrix with given dimensions, uses given rng for randomness.
    pub fn new_random<R: Rng + ?Sized>(rows: usize, cols: usize, rng: &mut R) -> Self
        where
            F: UniformRand,
    {
        let mut data = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for _ in 0..cols {
                row.push(F::rand(rng));
            }
            data.push(row);
        }
        Matrix { rows, cols, data }
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
        Matrix { rows, cols, data:field_data }
    }

    /// get the row at 0<idx<n
    pub fn row(&self, idx: usize) -> Vec<F>{
        assert!(idx < self.rows, "Row index out of bounds");
        self.data[idx].to_vec()
    }

    /// get mut the row at 0<idx<n
    pub fn row_mut(&mut self, idx: usize) -> &mut Vec<F>{
        assert!(idx < self.rows, "Row index out of bounds");
        &mut self.data[idx]
    }

    /// Print matrix
    pub fn pretty_print(&self) {
        for (i, shard) in self.data.iter().enumerate() {
            print!("row {:>2}: ", i);
            for &b in shard {
                print!("{:>3} ", b);
            }
            println!();
        }
    }
}


