use std::marker::PhantomData;
use anyhow::{anyhow, Result};
use ark_bls12_381::Bls12_381;
use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_poly::{DenseUVPolynomial, GeneralEvaluationDomain, Polynomial};
use ark_poly::univariate::DensePolynomial;
use reed_solomon_erasure::galois_8::ReedSolomon;
use crate::byte_data::{Data, Params};
use crate::traits::{DataMatrix, Encoder};
use ark_poly::domain::EvaluationDomain;
// use ark_poly_commit::Evaluations;
use ark_poly::Evaluations;
use crate::field_matrix::Matrix;


// ------------- G8 Encoder ------------

pub struct G8Encoder<T>{
    phantom_data: PhantomData<T>
}

impl G8Encoder<u8>{
    pub fn new() -> Self{
        Self{
            phantom_data: PhantomData::default()
        }
    }
}

impl Encoder<u8> for G8Encoder<u8> {
    type Params = Params;
    type DataMatrix<T> = Data<u8>;

    /// encode the columns of the data matrix in place
    fn encode(data: &mut Data<u8>) -> Result<()> {
        let n = data.params.n;
        assert!(data.params.k < n, "k must be less than total shards");
        let p = n - data.params.k;

        // ensure all rows are same length
        let row_size = data.matrix[0].len();
        for row in &data.matrix[1..] {
            assert_eq!(row.len(), row_size, "all rows must have equal length");
        }

        // build the encoder
        let rse = ReedSolomon::new(data.params.k, p)?;

        // prepare mutable slice references for in-place encode
        let mut shards_refs: Vec<&mut [u8]> = data.matrix.iter_mut()
            .map(|v| v.as_mut_slice())
            .collect();

        // encode
        rse.encode(&mut shards_refs)?;
        Ok(())
    }

    fn encode_col(data: &mut Data<u8>, c: usize) -> Result<()>{
        // bounds check
        if c >= data.params.m {
            return Err(anyhow!("col index {} out of bounds (< {})", c, data.params.m));
        }

        let n = data.params.n;
        let k = data.params.k;
        let p = n - k;

        // Build the column: data = existing byte, parity = zero
        let mut temp: Vec<Vec<u8>> = (0..n)
            .map(|i| {
                let byte = data.get(i,c).unwrap();
                if i < k {
                    vec![byte]
                } else {
                    vec![0u8]
                }
            })
            .collect();
        let mut refs: Vec<&mut [u8]> = temp.iter_mut().map(|v| v.as_mut_slice()).collect();

        // Encode that stripe
        let rse = ReedSolomon::new(k, p)?;
        rse.encode(&mut refs)?;

        // Write back parity
        for i in 0..n {
            let b = refs[i][0];
            if i >= k {
                data.set(i,c, b)?;
            }
        }

        Ok(())
    }

    fn reconstruct(params: Params, matrix_opts: &mut Vec<Option<Vec<u8>>>) -> Result<()>{
        let n = params.n;
        let k = params.k;
        let p = n - k;
        let rse = ReedSolomon::new(k, p).unwrap();
        // reconstruct missing rows
        rse.reconstruct(matrix_opts)?;
        Ok(())
    }
}

// ---------------- Bls12_381 Encoder -----------------

pub type E = Bls12_381;
pub type F = <E as Pairing>::ScalarField;
pub type UniPoly381 = DensePolynomial<F>;

pub struct BLSEncoder<T>{
    phantom_data: PhantomData<T>
}

impl BLSEncoder<u8>{
    pub fn new() -> Self{
        Self{
            phantom_data: PhantomData::default()
        }
    }
}

impl Encoder<u8> for BLSEncoder<u8> {
    type Params = Params;
    type DataMatrix<T> = Data<u8>;

    fn encode(data: &mut Self::DataMatrix<u8>) -> Result<()> {
        for i in 0..data.params.m {
            let _col = Self::encode_col(data, i)?;
        }
        Ok(())
    }

    fn encode_col(data: &mut Self::DataMatrix<u8>, c: usize) -> Result<()> {
        let n = data.params.n.clone();
        let k = data.params.k.clone();
        let mut col = data.get_col_mut(c);
        let col_f: Vec<F> = col.iter().map(|i| <F as PrimeField>::from_le_bytes_mod_order(&i.clone().to_le_bytes())).collect();
        let poly_poly = UniPoly381::from_coefficients_slice(&col_f);
        let domain: GeneralEvaluationDomain<F> = EvaluationDomain::<F>::new(n).unwrap();

        // let mut new_col: Vec<u8> = vec![];
        for i in k..n{
            let eval = poly_poly.evaluate(&domain.element(i));
            *col[i] = eval.0.0[0] as u8;
        }

        Ok(())
    }

    fn reconstruct(_params: Params, _matrix_opts: &mut Vec<Option<Vec<u8>>>) -> Result<()> {
        todo!()
    }
}

// --------- BLS Encoder over FieldMatrix ----------------

pub struct BLSFieldEncoder<T>{
    phantom_data: PhantomData<T>
}

impl Encoder<F> for BLSFieldEncoder<F>{
    type Params = Params;
    type DataMatrix<T> = Matrix<F>;

    fn encode(data: &mut Matrix<F>) -> Result<()> {
        for i in 0..data.params.m {
            let _col = Self::encode_col(data, i)?;
        }
        Ok(())
    }

    fn encode_col(data: &mut Matrix<F>, c: usize) -> Result<()> {
        let n = data.params.n.clone();
        let k = data.params.k.clone();
        let col: Vec<F> = data.get_col(c)?;

        let poly_domain: GeneralEvaluationDomain<F> = EvaluationDomain::<F>::new(n).ok_or(anyhow!("polycommit domain error"))?;
        let evals = Evaluations::from_vec_and_domain(col[0..k].to_vec(), poly_domain);
        let poly = evals.interpolate();

        for i in k..n{
                let eval = poly.evaluate(&poly_domain.element(i));
                data.set(i,c,eval)?;
        }
        Ok(())
    }

    fn reconstruct(_params: Params, _matrix_opts: &mut Vec<Option<Vec<F>>>) -> Result<()> {
        todo!()
    }
}