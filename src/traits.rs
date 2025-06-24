use anyhow::Result;

pub trait DataMatrix<T>{
    type Params;
    fn new_random(_: Self::Params) -> Self;
    fn update_col(&mut self, c: usize, new_col: &[T]);
    fn pretty_print(&self);
}

/// Encoder trait
pub trait Encoder<T>{

    /// encode in place the input data matrix
    fn encode(&mut self) -> Result<()>;
    /// encode a single column in place
    fn encode_col(&mut self, c: usize) -> Result<Vec<T>>;
    /// reconstruct in place
    fn reconstruct(&mut self) -> Result<()>;
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