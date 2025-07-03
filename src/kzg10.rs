use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_poly_commit::{
    LabeledPolynomial,
};
use ark_std::test_rng;
use anyhow::Result;
use ark_bls12_381::Bls12_381;
use ark_ec::pairing::Pairing;
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::{PrimeField, Zero};
use crate::traits::{CommitOutputTrait, PolyCommScheme};
use ark_poly_commit::kzg10::{KZG10, Proof, UniversalParams, Powers, VerifierKey, Commitment, Randomness};

pub type E = Bls12_381;
pub type F = <E as Pairing>::ScalarField;
pub type UniPoly381 = DensePolynomial<F>;
pub type PCS = KZG10<E, UniPoly381>;

pub type KZG10SRS = UniversalParams<E>;

pub struct KZG10PolyComm {}

pub struct KZG10CommitOutput {
    pub poly: LabeledPolynomial<F, UniPoly381>,
    pub comm: Commitment<E>,
    pub rand: Randomness<F, UniPoly381>,
}

impl KZG10CommitOutput {
    pub fn new(
        poly: LabeledPolynomial<F, UniPoly381>,
        comm: Commitment<E>,
        rand: Randomness<F, UniPoly381>,
    ) -> Self{
        Self{
            poly,
            comm,
            rand,
        }
    }
}


impl CommitOutputTrait for KZG10CommitOutput {
    type Poly = LabeledPolynomial<F, UniPoly381>;
    type Comm = Commitment<E>;
    type Rand = Randomness<F, UniPoly381>;

    fn get_poly(&self) -> &LabeledPolynomial<F, UniPoly381>{
        &self.poly
    }

    fn get_comm(&self) -> &Commitment<E>{
        &self.comm
    }

    fn get_rand(&self) -> &Randomness<F, UniPoly381>{
        &self.rand
    }
}

impl KZG10PolyComm{
    fn commit_single(srs: &KZG10SRS, input: F, index: usize) -> Result<Commitment<E>> {
        let power = &srs.powers_of_g[index];

        let c = power.mul_bigint(input.into_bigint());

        Ok(
            Commitment::<E>(c.into_affine())
        )
    }
}

impl PolyCommScheme<F> for KZG10PolyComm {
    type SRS = KZG10SRS;
    type VK = VerifierKey<E>;
    type CommitOutput = KZG10CommitOutput;
    type Comm = Commitment<E>;
    type Proof = Proof<E>;

    fn setup(degree: usize) -> Result<Self::SRS> {
        let rng = &mut test_rng();
        let pp = PCS::setup(degree,false, rng)?;
        Ok(pp)
    }

    fn commit(srs: &Self::SRS, input: Vec<F>) -> Result<Self::CommitOutput> {
        let rng = &mut test_rng();
        let degree = input.len();
        let powers = get_powers(&srs, degree)?;

        // input are poly coeffs
        let input_poly = DensePolynomial::<F>::from_coefficients_vec(input);
        let label = String::from("row_poly");
        let labeled_poly = LabeledPolynomial::new(
            label,
            input_poly,
            Some(degree),
            None,
        );

        let (comm, rand) = PCS::commit(&powers, &labeled_poly, None, Some(rng))?;

        Ok(
            KZG10CommitOutput::new(labeled_poly, comm, rand)
        )
    }

    fn update_commitment(srs: &Self::SRS, original_comm: &mut Self::CommitOutput, original_cell: F, new_cell:F, index: usize) -> Result<()> {
        // check if there is difference,
        if new_cell.clone() - original_cell.clone() == F::zero() {
            return Ok(())
        }

        // commit to original and new cells
        let original_cell_comm = Self::commit_single(srs, original_cell, index)?;
        let new_cell_comm = Self::commit_single(srs, new_cell.clone(), index)?;

        // compute delta
        let delta_comm = (new_cell_comm.0-original_cell_comm.0).into_affine();
        // update the commitment
        let mut tmp = original_comm.comm.0.clone().into_group();
        tmp += &delta_comm;
        original_comm.comm.0 = tmp.into_affine();
        // update the poly
        let original_poly = original_comm.poly.polynomial_mut();
        original_poly.coeffs[index] = new_cell;
        // no update to rand because we assume it is empty i.e. no hiding
        Ok(())
    }

    fn open(
            comm: &KZG10CommitOutput,
            srs: &KZG10SRS,
            point: F,
    ) -> Result<Self::Proof> {

        // powers from the srs
        let m = srs.powers_of_g.len();
        let powers= get_powers(&srs, m)?;

        // get row poly and rand
        let poly     = &comm.poly;
        let rand    = &comm.rand;

        let proof = PCS::open(
            &powers,
            poly,
            point,
            rand,
        )?;

        Ok(proof)
    }

    fn verify(
        vk:   &Self::VK,
        comm: &Self::Comm,
        point: F,
        value: F,
        proof: &Self::Proof,
    ) -> Result<bool> {

        Ok( PCS::check(
            &vk,
            comm,
            point,
            value,
            proof,
        )? )
    }
}

// --------------- Utils -----------------

/// get `degree` number of powers from the universal params
fn get_powers(
    pp: &UniversalParams<E>,
    degree: usize,
) -> Result<Powers<E>> {
    let powers_of_g = pp.powers_of_g[..=degree].to_vec();
    let powers_of_gamma_g = (0..=degree)
        .map(|i| pp.powers_of_gamma_g[&i])
        .collect();
    let powers = Powers {
        powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
        powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
    };
    Ok(powers)
}

pub fn get_vk(
    pp: &UniversalParams<E>,
) -> Result<VerifierKey<E>> {
    let vk = VerifierKey {
        g: pp.powers_of_g[0],
        gamma_g: pp.powers_of_gamma_g[&0],
        h: pp.h,
        beta_h: pp.beta_h,
        prepared_h: pp.prepared_h.clone(),
        prepared_beta_h: pp.prepared_beta_h.clone(),
    };
    Ok(vk)
}