use std::marker::PhantomData;
use anyhow::Result;
use crate::byte_data::Params;

pub trait DataMatrix<T>{
    type Params;
    fn new_random(params: Self::Params) -> Self;
    fn get(&self, r: usize, c: usize) -> Result<T>;
    fn get_row(&self, r: usize) -> Result<Vec<T>>;
    fn get_col(&self, c: usize) -> Result<Vec<T>>;
    fn set(&mut self, r: usize, c: usize, elem: T) -> Result<()>;
    fn update_col(&mut self, c: usize, new_col: &[T]) -> Result<()>;
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
    fn encode_col(data: &mut Self::DataMatrix<T>, c: usize) -> Result<()>;
    /// reconstruct in place
    fn reconstruct(params: Params, matrix_opts: &mut Vec<Option<Vec<T>>>) -> Result<()>;
}

pub trait CommitOutputTrait {
    type Poly;
    type Comm;
    type Rand;

    fn get_poly(&self) -> &Self::Poly;

    fn get_comm(&self) -> &Self::Comm;

    fn get_rand(&self) -> &Self::Rand;
}

// pub trait SRSTrait<F>{
//     // public/universal params
//     type PP;
//     // domain type
//     type Domain;
//
//     fn get_pp(&self) -> &Self::PP;
//     fn get_domain(&self) -> &Self::Domain;
//     fn get_domain_element(&self, idx: usize) -> F;
//     fn get_domain_size(&self) -> usize;
// }

/// Polynomial Commitment scheme (e.g. KZG) trait
pub trait PolyCommScheme<F>{
    type SRS;
    type VK;
    type CommitOutput: CommitOutputTrait;
    type Comm;
    type Proof;

    fn setup(degree: usize) -> Result<Self::SRS>;
    fn commit(srs: &Self::SRS, input:Vec<F>) -> Result<Self::CommitOutput>;
    fn update_commitment(srs: &Self::SRS, original_comm: &mut Self::CommitOutput, original_cell: F, new_cell:F, index: usize) -> Result<()>;
    fn open(
        comm: &Self::CommitOutput,
        srs: &Self::SRS,
        point: F
    ) -> Result<Self::Proof>;
    fn verify(
        vk:   &Self::VK,
        comm: &Self::Comm,
        point: F,
        value: F,
        proof: &Self::Proof,
    ) -> Result<bool>;
}

/// Polynomial Commitment scheme for a field Matrix
/// it commits to the rows of the Matrix
/// and allows updating the row commitments
pub trait MatrixPolyCommScheme<F, P:PolyCommScheme<F>>{
    type FieldMatrix: DataMatrix<F>;

    fn setup(m: usize) -> Result<P::SRS>;
    fn commit(srs: &P::SRS, matrix:&Self::FieldMatrix) -> Result<MatrixCommitOutput<F, P>>;
    fn update_commitments(
        srs: &P::SRS,
        comm: &mut MatrixCommitOutput<F, P>,
        col_idx: usize,
        old_col: &[F],
        new_col: &[F],
    ) -> Result<()>;
    fn open(
        comm: &MatrixCommitOutput<F, P>,
        srs: &P::SRS,
        row: usize,
        point: F,
    ) -> Result<P::Proof>;
    fn verify(
        vk:   &P::VK,
        comm: &P::Comm,
        point: F,
        value: F,
        proof: &P::Proof,
    ) -> Result<bool>;
}

pub struct MatrixCommitOutput<F, P: PolyCommScheme<F>> {
    pub comm_output: Vec<P::CommitOutput>,
    phantom_data: PhantomData<F>
}

impl<F, P: PolyCommScheme<F>> MatrixCommitOutput<F, P> {
    pub fn new(
        comm_output: Vec<P::CommitOutput>
    ) -> Self{
        Self{
            comm_output,
            phantom_data:PhantomData::default(),
        }
    }

    pub fn get_poly(&self, idx: usize) -> &<P::CommitOutput as CommitOutputTrait>::Poly{
        &self.comm_output[idx].get_poly()
    }

    pub fn get_comm(&self, idx: usize) -> &<P::CommitOutput as CommitOutputTrait>::Comm{
        &self.comm_output[idx].get_comm()
    }

    pub fn get_rand(&self, idx: usize) -> &<P::CommitOutput as CommitOutputTrait>::Rand{
        &self.comm_output[idx].get_rand()
    }
}