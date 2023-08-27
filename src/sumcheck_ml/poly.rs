use std::marker::PhantomData;
use ark_ff::Field;
use bitvec::slice::BitSlice;
use ark_poly::{
    multivariate::{self, SparseTerm},
    DenseMVPolynomial,
     Polynomial,
};
/// An error type of sum check protocol
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("prover claim mismatches evaluation {0} {1}")]
    ProverClaimMismatch(String, String),

    #[error("verifier has no oracle access to the polynomial")]
    NoPolySet,
}

pub struct BooleanHypercube<F: Field> {
    n: u32,
    current: u64,
    __f: PhantomData<F>,
}

impl<F: Field> BooleanHypercube<F> {

    pub fn new(n: u32) -> Self {
        Self {
            n,
            current: 0,
            __f: PhantomData,
        }
    }
}

impl<F: Field> Iterator for BooleanHypercube<F> {
    type Item = Vec<F>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == 2u64.pow(self.n) {
            None
        } else {
            let vec = self.current.to_le_bytes();
            let s: &BitSlice<u8> = BitSlice::try_from_slice(&vec).unwrap();
            self.current += 1;

            Some(
                s.iter()
                    .take(self.n as usize)
                    .map(|f| match *f {
                        false => F::zero(),
                        true => F::one(),
                    })
                    .collect(),
            )
        }
    }
}

pub trait SumCheckPolynomial<F: Field> {
    /// Evaluates `self` at a given point
    fn evaluate(&self, point: &[F]) -> Option<F>;

    /// Returns the number of variables in `self`
    fn num_vars(&self) -> usize;

    /// Returns a list of evaluations over the entire BooleanHypercube domain
    fn to_evaluations(&self) -> Vec<F>;
}

impl<F: Field> SumCheckPolynomial<F> for multivariate::SparsePolynomial<F, SparseTerm> {
    fn evaluate(&self, point: &[F]) -> Option<F> {
        Some(Polynomial::evaluate(self, &point.to_owned()))
    }

    fn num_vars(&self) -> usize {
        DenseMVPolynomial::num_vars(self)
    }

    fn to_evaluations(&self) -> Vec<F> {
        BooleanHypercube::new(DenseMVPolynomial::num_vars(self) as u32)
            .map(|point| Polynomial::evaluate(self, &point))
            .collect()
    }
	
}
