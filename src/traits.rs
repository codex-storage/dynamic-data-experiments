use anyhow::Result;
use crate::byte_data::Params;

pub trait DataMatrix<T>{
    type Params;
    fn new_random(params: Self::Params) -> Self;
    fn update_col(&mut self, c: usize, new_col: &[T]);
    fn pretty_print(&self);
}

/// Encoder trait
pub trait Encoder<T>{
    type Params;
    /// data matrix type to encode
    type DataMatrix<U>;

    /// encode in place the input data matrix
    fn encode(data: &mut Self::DataMatrix<T>) -> Result<()>;
    /// encode a single column in place
    fn encode_col(data: &mut Self::DataMatrix<T>, c: usize) -> Result<Vec<T>>;
    /// reconstruct in place
    fn reconstruct(params: Params, matrix_opts: &mut Vec<Option<Vec<T>>>) -> Result<()>;
}

/// Polynomial Commitment scheme (e.g. KZG) trait
pub trait PolynomialCommitmentScheme{
    type Params;
    type Field;
    type FieldMatrix<F>;
    type SRS;
    type Commitment;
    type Proof;

    fn new(_params: Self::Params) -> Self;
    fn setup(&self) -> Result<Self::SRS>;
    fn commit(&self, _srs: &Self::SRS, _matrix:&Self::FieldMatrix<Self::Field>) -> Result<Self::Commitment>;
    fn update_commitments(
        srs: &Self::SRS,
        comm: &mut Self::Commitment,
        row_idx: usize,
        old_row: &[Self::Field],
        new_row: &[Self::Field],
    ) -> Result<()>;
    fn open(
        _: &Self::Commitment,
        _: &Self::SRS,
        _row: usize,
        _col: usize,
    ) -> Result<Self::Proof>;
    fn batch_open(
        _: &Self::Commitment,
        _: &Self::SRS,
        _rows: Vec<usize>,
        _cols: Vec<usize>,
    ) -> Result<Vec<Self::Proof>>;
    fn verify(
        comms: &Self::Commitment,
        srs:   &Self::SRS,
        row:   usize,
        col:   usize,
        value: Self::Field,
        proof: &Self::Proof,
    ) -> Result<bool>;
    fn batch_verify(
        comms: &Self::Commitment,
        srs:   &Self::SRS,
        rows:   Vec<usize>,
        cols:   Vec<usize>,
        values: Vec<Self::Field>,
        proof: &Vec<Self::Proof>,
    ) -> Result<bool>;
}