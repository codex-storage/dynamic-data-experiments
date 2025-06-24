use ark_ff::{One, PrimeField};
use std::ops::{Add, AddAssign};
use ark_poly::univariate::DensePolynomial;
use ark_poly::{EvaluationDomain, Evaluations, GeneralEvaluationDomain};
use ark_poly_commit::{
    PolynomialCommitment,
    LabeledPolynomial,
    LabeledCommitment,
    marlin_pc::{ Commitment, Randomness},
};
use ark_std::{end_timer, start_timer, test_rng};
use anyhow::{anyhow, Result};
use ark_bls12_381::Bls12_381;
use ark_crypto_primitives::sponge::CryptographicSponge;
use ark_ec::CurveGroup;
use ark_ec::pairing::Pairing;
use ark_poly_commit::marlin_pc::MarlinKZG10;
use ark_poly_commit::sonic_pc::UniversalParams;
use crate::byte_data::Params;
use crate::field_matrix::Matrix;
use crate::traits::PolynomialCommitmentScheme;
use ark_crypto_primitives::sponge::poseidon::{PoseidonSponge, PoseidonConfig};
use ark_poly_commit::kzg10::Proof;

pub type E = Bls12_381;
pub type F = <E as Pairing>::ScalarField;
pub type UniPoly381 = DensePolynomial<F>;
pub type PCS = MarlinKZG10<E, UniPoly381>;

pub struct KZGSRS{
    pub ploycommit_domain: GeneralEvaluationDomain<F>,
    pub pp: UniversalParams<E>
}

pub struct KZGPolyComm{
    params: Params,
}

impl PolynomialCommitmentScheme for KZGPolyComm {
    type Params = Params;
    type Field = F;
    type FieldMatrix<F> = Matrix<Self::Field>;
    type SRS = KZGSRS;
    type Commitment = KZGCommitments;
    type Proof = Proof<E>;

    fn new(params: Params) -> Self{
        Self{
            params,
        }
    }

    fn setup(&self) -> Result<Self::SRS> {
        let rng = &mut test_rng();
        let pp = PCS::setup(self.params.m,None, rng)?;
        let ploycommit_domain = EvaluationDomain::<F>::new(self.params.m).ok_or(anyhow!("polycommit domain error"))?;
        Ok(KZGSRS{
            ploycommit_domain,
            pp,
        })
    }

    fn commit(&self, srs: &Self::SRS, matrix: &Self::FieldMatrix<F>) -> Result<Self::Commitment> {
        let rng = &mut test_rng();
        let degree = self.params.m;
        let (ck, _vk) = PCS::trim(&srs.pp, degree, degree, Some(&[degree]))?;
        let mut row_polynomials = vec![];
        let timer = start_timer!(|| format!("Poly evaluations and interpolation for {} rows", degree));
        for i in 0..matrix.rows{
            let poly_evals = Evaluations::from_vec_and_domain(matrix.row(i), srs.ploycommit_domain.clone());
            let row_poly = poly_evals.interpolate();
            let label = String::from(format!("row_poly_{}", i));
            let labeled_poly = LabeledPolynomial::new(
                label,
                row_poly,
                Some(degree),
                Some(degree),
            );
            row_polynomials.push(labeled_poly);
        }
        end_timer!(timer);
        let timer = start_timer!(|| format!("KZG commitment for {} columns", degree));
        let (labeled_comms, states) = PCS::commit(&ck, &row_polynomials, Some(rng)).unwrap();
        end_timer!(timer);
        Ok(
            KZGCommitments::new(row_polynomials, labeled_comms, states)
        )
    }

    fn open(comms: &Self::Commitment, srs: &Self::SRS, row: usize, col: usize) -> Result<Self::Proof> {
        // point
        let z = srs.ploycommit_domain.element(col);

        // trim the srs
        let m = srs.ploycommit_domain.size();
        let (ck, _vk) = PCS::trim(&srs.pp, m, m, Some(&[m]))?;

        let (polys, comms_vec, states) = comms.get_refs();
        let poly     = &polys[row];
        let commit   = &comms_vec[row];
        let state    = &states[row];

        let mut sponge = test_sponge::<F>();

        let proof = PCS::open(
            &ck,
            std::iter::once(poly),
            std::iter::once(commit),
            &z,
            &mut sponge,
            &mut std::iter::once(state),
            None,
        )?;

        Ok(proof)
    }

    fn batch_open(_: &Self::Commitment, _: &Self::SRS, _rows: Vec<usize>, _cols: Vec<usize>) -> Result<Vec<Self::Proof>> {
        todo!()
    }

    fn verify(
        comms: &Self::Commitment,
        srs:   &Self::SRS,
        row:   usize,
        col:   usize,
        value: F,
        proof: &Self::Proof,
    ) -> Result<bool> {
        let z = srs.ploycommit_domain.element(col);

        let m = srs.ploycommit_domain.size();
        let (_ck, vk) = PCS::trim(&srs.pp, m, m, Some(&[m]))?;

        // get labeled commitment
        let (_polys, commits, _states) = comms.get_refs();
        let commit = &commits[row];

        let mut sponge = test_sponge::<F>();
        Ok( PCS::check(
            &vk,
            std::iter::once(commit),
            &z,
            std::iter::once(value),
            proof,
            &mut sponge,
            None,
        )? )
    }

    fn batch_verify(_comms: &Self::Commitment, _srs: &Self::SRS, _rows: Vec<usize>, _cols: Vec<usize>, _values: Vec<F>, _proof: &Vec<Self::Proof>) -> Result<bool> {
        todo!()
    }

    fn update_commitments(
        srs: &KZGSRS,
        comm: &mut KZGCommitments,
        row_idx: usize,
        old_row: &[F],
        new_row: &[F],
    ) -> Result<()> {

        let n = comm.poly.len();
        let domain = &srs.ploycommit_domain;
        let m = domain.size();
        let (ck, _vk) = PCS::trim(&srs.pp, m, 2, Some(&[m]))?;
        // Bounds and length checks
        assert!(row_idx < n, "row_idx {} out of bounds ({} rows)", row_idx, n);
        assert_eq!(old_row.len(), m, "old_row must have length {}", m);
        assert_eq!(new_row.len(), m, "new_row must have length {}", m);

        let deltas: Vec<F> = old_row.iter()
            .zip(new_row.iter())
            .map(|(o, n)| *n - *o)
            .collect();

        let delta_poly: DensePolynomial<F> =
            Evaluations::from_vec_and_domain(deltas, domain.clone())
                .interpolate();

        let label = format!("row_diff_{}", row_idx);
        let labeled = LabeledPolynomial::new(label, delta_poly.clone(), Some(m), None);
        let rng = &mut test_rng();
        let (diff_comms, diff_rands) = PCS::commit(&ck, std::iter::once(&labeled), Some(rng))?;
        let diff_comm = &diff_comms[0];
        let diff_rand = &diff_rands[0];

        let f_row = comm.poly[row_idx].polynomial_mut();
        f_row.add_assign(&delta_poly);

        let mut cmt = comm.comm[row_idx].commitment().clone();
        let main_patch = diff_comm.commitment().comm.0;
        cmt.comm.0 = cmt.comm.0.add(&main_patch).into_affine();
        if let (Some(mut shifted), Some(diff_shifted)) = (
            cmt.shifted_comm.clone(),
            diff_comm.commitment().shifted_comm.clone(),
        ) {
            shifted.0 = shifted.0.add(&diff_shifted.0).into_affine();
            cmt.shifted_comm = Some(shifted);
        }
        let lbl = comm.comm[row_idx].label().to_string();
        let dgb = comm.comm[row_idx].degree_bound();
        comm.comm[row_idx] = LabeledCommitment::new(lbl, cmt, dgb);

        comm.rand[row_idx].add_assign((F::one(), diff_rand));

        Ok(())
    }
}

fn test_sponge<F: PrimeField>() -> PoseidonSponge<F> {
    let full_rounds = 8;
    let partial_rounds = 31;
    let alpha = 17;

    let mds = vec![
        vec![F::one(), F::zero(), F::one()],
        vec![F::one(), F::one(), F::zero()],
        vec![F::zero(), F::one(), F::one()],
    ];

    let mut v = Vec::new();
    let mut ark_rng = test_rng();

    for _ in 0..(full_rounds + partial_rounds) {
        let mut res = Vec::new();

        for _ in 0..3 {
            res.push(F::rand(&mut ark_rng));
        }
        v.push(res);
    }
    let config = PoseidonConfig::new(full_rounds, partial_rounds, alpha, mds, v, 2, 1);
    PoseidonSponge::new(&config)
}

pub struct KZGCommitments{
    pub poly: Vec<LabeledPolynomial<F, UniPoly381>>,
    pub comm: Vec<LabeledCommitment<Commitment<E>>>,
    pub rand: Vec<Randomness<F, UniPoly381>>,
}

impl KZGCommitments {
    pub fn new(
        poly: Vec<LabeledPolynomial<F, UniPoly381>>,
        comm: Vec<LabeledCommitment<Commitment<E>>>,
        rand: Vec<Randomness<F, UniPoly381>>,
    ) -> Self{
        Self{
            poly,
            comm,
            rand,
        }
    }
    pub fn get_refs(&self) ->(
        &Vec<LabeledPolynomial<F, UniPoly381>>,
        &Vec<LabeledCommitment<Commitment<E>>>,
        &Vec<Randomness<F, UniPoly381>>,
    ){
        (&self.poly, &self.comm, &self.rand)
    }
}