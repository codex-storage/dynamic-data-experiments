use std::marker::PhantomData;
use anyhow::Result;
use ark_ff::Field;
use crate::field_matrix::Matrix;
use crate::traits::{MatrixPolyCommScheme, DataMatrix, PolyCommScheme, MatrixCommitOutput};


pub struct MatrixPolyComm<F, P: PolyCommScheme<F>> {
    phantom_data: PhantomData<(F,P)>
}

impl<F: Field + Clone, P: PolyCommScheme<F>> MatrixPolyCommScheme<F, P> for MatrixPolyComm<F, P> {
    type FieldMatrix = Matrix<F>;

    /// setup takes `m`=`number of columns` in the matrix
    fn setup(m: usize) -> Result<P::SRS> {
        P::setup(m)
    }

    fn commit(srs: &P::SRS, matrix: &Self::FieldMatrix) -> Result<MatrixCommitOutput<F, P>> {

        let mut row_comm_output = vec![];
        for i in 0..matrix.params.n{
            let row = matrix.get_row(i)?;
            let output = P::commit(srs,row)?;
            row_comm_output.push(output);
        }

        Ok(
            MatrixCommitOutput::new(row_comm_output)
        )
    }

    /// updates the row commitments after updating/modifying columns
    /// since the data DataMatrix should only allow column updates
    /// and since we commit to rows
    /// this means we update all row commitments that are affected by the data matrix update
    fn update_commitments(
        srs: &P::SRS,
        comm: &mut MatrixCommitOutput<F, P>,
        col_idx: usize,
        old_col: &[F],
        new_col: &[F],
    ) -> Result<()> {
        // check input is consistent
        assert_eq!(old_col.len(), new_col.len(), "col sizes don't match");

        // loop through all new_col elements to see if there is an update at each cell
        // if there is, then update the commitment
        for r in 0..new_col.len(){
            let original_cell: F = old_col[r].clone();
            let new_cell: F = new_col[r].clone();
            P::update_commitment(srs, &mut comm.comm_output[r], original_cell, new_cell, col_idx)?;
        }

        Ok(())
    }

    fn open(comm: &MatrixCommitOutput<F, P>, srs: &P::SRS, row: usize, point: F) -> Result<P::Proof> {

        let proof = P::open(&comm.comm_output[row], srs, point)?;

        Ok(proof)
    }

    fn verify(
        vk:   &P::VK,
        comm: &P::Comm,
        point: F,
        value: F,
        proof: &P::Proof,
    ) -> Result<bool> {

        Ok( P::verify(
            &vk,
            comm,
            point,
            value,
            proof,
        )? )
    }

}

