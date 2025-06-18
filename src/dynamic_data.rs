use ark_bls12_381::Bls12_381;
use ark_ec::pairing::Pairing;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Evaluations, EvaluationDomain, GeneralEvaluationDomain};
use ark_poly_commit::{Polynomial, marlin_pc::MarlinKZG10, LabeledPolynomial, PolynomialCommitment, QuerySet, LabeledCommitment};
use ark_poly_commit::marlin_pc::{Commitment, Randomness,};
use anyhow::{anyhow, Result};
use ark_poly_commit::sonic_pc::UniversalParams;
use ark_std::{end_timer, start_timer, test_rng};
use crate::matrix::Matrix;

type E = Bls12_381;
type F = <E as Pairing>::ScalarField;
type UniPoly381 = DensePolynomial<F>;
type PCS = MarlinKZG10<E, UniPoly381>;

// struct for the dynamic data scheme,
// contains the params and functions needed for the dynamic data scheme
pub struct DynamicData{
    n: usize, // the row size of the data matrix - un-coded
    k: usize, // the row size of the erasure coded data matrix
    m: usize, // the column size of the matrix

    ploycommit_domain: GeneralEvaluationDomain<F>,
    encoding_domain: GeneralEvaluationDomain<F>,

    pp: UniversalParams<Bls12_381>
}

impl DynamicData {
    // setup the dynamic data scheme
    pub fn setup(n: usize, k:usize, m:usize) -> Result<Self>{
        let rng = &mut test_rng();
        let pp = PCS::setup(m,None, rng)?;
        let ploycommit_domain = EvaluationDomain::<F>::new(m).ok_or(anyhow!("polycommit domain error"))?;
        let encoding_domain = EvaluationDomain::<F>::new(n).ok_or(anyhow!("encoding domain error"))?;
        Ok(Self{
            n,
            k,
            m,
            ploycommit_domain,
            encoding_domain,
            pp,
        })
    }

    pub fn commit_columns(&self, matrix: Matrix<F>) -> Result<(
        Vec<LabeledPolynomial<F, UniPoly381>>,
        Vec<LabeledCommitment<Commitment<E>>>,
        Vec<Randomness<F, UniPoly381>>,
    )>{
        let rng = &mut test_rng();
        let degree = self.m;
        let (ck, vk) = PCS::trim(&self.pp, degree, 2, Some(&[degree])).unwrap();
        let mut col_polynomials = vec![];
        let timer = start_timer!(|| format!("Poly evaluations and interpolation for {} columns", degree));
        for i in 0..matrix.cols(){
            let poly_evals = Evaluations::from_vec_and_domain(matrix.column(i), self.ploycommit_domain.clone());
            let col_poly = poly_evals.interpolate();
            let label = String::from(format!("column_poly_{}", i));
            let labeled_poly = LabeledPolynomial::new(
                label,
                col_poly,
                Some(degree),
                Some(2),
            );
            col_polynomials.push(labeled_poly);
        }

        let timer = start_timer!(|| format!("KZG commitment for {} columns", degree));
        let (labeled_comms, states) = PCS::commit(&ck, &col_polynomials, Some(rng)).unwrap();
        end_timer!(timer);
        Ok((col_polynomials,labeled_comms, states))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::test_rng;

    #[test]
    fn test_commit_columns_roundtrip() {
        // dimensions: 3 rows, 2 columns
        let n = 8;
        let k = 4;
        let m = 8;

        // setup
        let rng = &mut test_rng();
        let dd = DynamicData::setup(n, k, m).expect("setup should succeed");

        // make a random n×m matrix
        let matrix = Matrix::new_random(n, m, rng);

        // commit to its columns
        let (col_polys, commitments, randomness) =
            dd.commit_columns(matrix.clone()).expect("commit_columns should succeed");

        // we produced exactly one polynomial, one comm, one rand per column
        assert_eq!(col_polys.len(), m);
        assert_eq!(commitments.len(), m);
        assert_eq!(randomness.len(), m);

        // check that each polynomial really interpolates its original column
        for (i, poly) in col_polys.iter().enumerate() {
            let col = matrix.column(i);
            // evaluate poly at each domain point and collect
            let evals: Vec<_> = dd
                .ploycommit_domain
                .elements()
                .map(|x| poly.polynomial().evaluate(&x))
                .collect();
            assert_eq!(evals, col);
        }
    }
}
