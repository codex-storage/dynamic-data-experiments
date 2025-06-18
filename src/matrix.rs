use ark_ff::Field;
use std::ops::{Index, IndexMut};
use ark_std::rand::Rng;
use ark_std::UniformRand;


/// a generic dense matrix stored in row-major order.
#[derive(Clone, Debug, PartialEq)]
pub struct Matrix<T: Field + Clone> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T: Field + Clone> Matrix<T> {
    /// Creates a new matrix from raw data.
    pub fn new(rows: usize, cols: usize, data: Vec<T>) -> Self {
        assert!(data.len() == rows * cols, "Data length must equal rows*cols");
        Matrix { rows, cols, data }
    }

    /// Generates a random matrix with given dimensions, uses given rng for randomness.
    pub fn new_random<R: Rng + ?Sized>(rows: usize, cols: usize, rng: &mut R) -> Self
        where
            T: UniformRand,
    {
        let mut data = Vec::with_capacity(rows * cols);
        for _ in 0..rows * cols {
            data.push(T::rand(rng));
        }
        Matrix { rows, cols, data }
    }

    /// Creates a zero matrix (rows x cols).
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Matrix { rows, cols, data: vec![T::zero(); rows * cols] }
    }

    /// Creates an identity matrix of size n x n.
    pub fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n, n);
        for i in 0..n {
            m[(i, i)] = T::one();
        }
        m
    }

    /// Constructs from a nested Vec
    pub fn from_nested_vec(nested: Vec<Vec<T>>) -> Self {
        let rows = nested.len();
        assert!(rows > 0, "must have at least one row");
        let cols = nested[0].len();
        let mut data = Vec::with_capacity(rows * cols);
        for row in nested.into_iter() {
            assert!(row.len() == cols, "all rows must have the same length");
            data.extend(row);
        }
        Matrix { rows, cols, data }
    }

    /// Returns the number of rows.
    #[inline]
    pub fn rows(&self) -> usize { self.rows }

    /// Returns the number of columns.
    #[inline]
    pub fn cols(&self) -> usize { self.cols }

    /// Returns both dimensions (rows, cols).
    #[inline]
    pub fn dims(&self) -> (usize, usize) { (self.rows, self.cols) }

    /// Returns a reference to the element at (row, col). Panics if out of bounds.
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> &T {
        assert!(row < self.rows && col < self.cols, "Index out of bounds");
        &self.data[row * self.cols + col]
    }

    /// Returns a mutable reference to the element at (row, col).
    #[inline]
    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut T {
        assert!(row < self.rows && col < self.cols, "Index out of bounds");
        &mut self.data[row * self.cols + col]
    }

    /// Returns a slice for the given row.
    pub fn row(&self, row: usize) -> &[T] {
        assert!(row < self.rows, "Row index out of bounds");
        let start = row * self.cols;
        &self.data[start..start + self.cols]
    }

    /// Returns a mutable slice for the given row.
    pub fn row_mut(&mut self, row: usize) -> &mut [T] {
        assert!(row < self.rows, "Row index out of bounds");
        let start = row * self.cols;
        &mut self.data[start..start + self.cols]
    }

    /// Swaps two rows in-place.
    pub fn swap_rows(&mut self, i: usize, j: usize) {
        assert!(i < self.rows && j < self.rows, "Row index out of bounds");
        for col in 0..self.cols {
            let a = i * self.cols + col;
            let b = j * self.cols + col;
            self.data.swap(a, b);
        }
    }

    /// Horizontal concatenation: [self | other].
    pub fn hcat(&self, other: &Self) -> Self {
        assert!(self.rows == other.rows, "Row counts must match");
        let mut result = Self::zeros(self.rows, self.cols + other.cols);
        for r in 0..self.rows {
            // copy self
            let src = r * self.cols;
            let dst = r * result.cols;
            result.data[dst..dst + self.cols]
                .copy_from_slice(&self.data[src..src + self.cols]);
            // copy other
            let src2 = r * other.cols;
            result.data[dst + self.cols..dst + self.cols + other.cols]
                .copy_from_slice(&other.data[src2..src2 + other.cols]);
        }
        result
    }

    /// Selects a subset of columns by index.
    pub fn select_columns(&self, cols_idx: &[usize]) -> Self {
        let mut result = Self::zeros(self.rows, cols_idx.len());
        for r in 0..self.rows {
            for (j, &c) in cols_idx.iter().enumerate() {
                result.data[r * result.cols + j] = self.get(r, c).clone();
            }
        }
        result
    }

    /// Returns a Vec of all elements in the given column.
    pub fn column(&self, col: usize) -> Vec<T> {
        assert!(col < self.cols, "Column index out of bounds");
        let mut v = Vec::with_capacity(self.rows);
        for r in 0..self.rows {
            v.push(self.get(r, col).clone());
        }
        v
    }

    /// Computes the inverse via in-place Gauss–Jordan; returns None if singular.
    pub fn invert(&self) -> Option<Self> {
        assert!(self.rows == self.cols, "Can only invert square matrices");
        let n = self.rows;
        let mut aug = self.hcat(&Self::identity(n));

        for i in 0..n {
            // pivot check and swap if zero
            if aug[(i, i)].is_zero() {
                if let Some(k) = (i + 1..n).find(|&k| !aug[(k, i)].is_zero()) {
                    aug.swap_rows(i, k);
                } else {
                    return None;
                }
            }
            // normalize pivot row
            let inv_pivot = aug[(i, i)].inverse().unwrap();
            for col in i..2 * n {
                let idx = i * aug.cols + col;
                aug.data[idx] = aug.data[idx].clone() * inv_pivot.clone();
            }
            // Clone pivot row slice
            let pivot_start = i * aug.cols + i;
            let pivot_len = 2 * n - i;
            let pivot_row: Vec<T> = aug.data[pivot_start..pivot_start + pivot_len].to_vec();

            // remove other rows
            for r in 0..n {
                if r != i {
                    let factor = aug[(r, i)].clone();
                    if !factor.is_zero() {
                        let row_offset = r * aug.cols;
                        for k in 0..pivot_len {
                            let idx = row_offset + i + k;
                            aug.data[idx] = aug.data[idx].clone() - factor.clone() * pivot_row[k].clone();
                        }
                    }
                }
            }
        }
        Some(aug.select_columns(&(n..2 * n).collect::<Vec<_>>()))
    }
}

// indexing with (row, col)
impl<T: Field + Clone> Index<(usize, usize)> for Matrix<T> {
    type Output = T;
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        self.get(row, col)
    }
}

// mutable indexing with (row, col)
impl<T: Field + Clone> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        self.get_mut(row, col)
    }
}
