use ark_ff::Field;
use crate::sumcheck_ml::poly::*;
use ark_std::{rand::Rng};

pub trait RngF<F> {
    fn draw(&mut self) -> F;
}

impl<F: Field, T: Rng> RngF<F> for T {
    fn draw(&mut self) -> F {
		let t = F::rand(self);
		// println!("RngF::draw()# {:?} {:?}",&t,&t.to_string());
        t
    }
}


/// The state of the Verifier.
pub struct Verifier<F: Field, P: SumCheckPolynomial<F>> {
    n: usize,// Number of variables in the original polynomial.
    claim: F,// A $claim$ value claimed by the Prover.
    r: Vec<F>,// Previously picked random values $r_1,...,r_{j-1}$.
    g: Option<P>,// Original polynomial for oracle access
    pub expect:F
}

/// Values returned by Validator as a result of its run on every step.
#[derive(Debug)]
pub enum VerifierRoundResult<F: Field> {
    JthRound(F),
    FinalRound(bool),
}

impl<F: Field, P: SumCheckPolynomial<F>> Verifier<F, P> {
    /// Create the new state of the [`Verifier`].
    /// $n$ - degree of the polynomial
    /// $claim$ - the value claimed to be true answer by the [`Prover`].
    pub fn new(g: Option<P>,claim: F) -> Self {
		let num_vars = g.as_ref().unwrap().num_vars();
        Self {
            n:num_vars,
            claim,
            r: Vec::with_capacity(num_vars),
            g,
            expect:claim
        }
    }

    /// Perform the $j$-th round of the [`Verifier`] side of the protocol.
    pub fn round<R: RngF<F>>(
        &mut self,
        p0:F,p1:F,
        rng: &mut R,
    ) -> Result<VerifierRoundResult<F>, Error> {
		let r_j = rng.draw();

        if self.expect != p0 + p1 {
            Err(Error::ProverClaimMismatch(
                format!("{:?}", self.expect),
                format!("{:?} {:?}", p0 , p1),
            ))
        } else if self.r.len() == (self.n - 1) {
            // Last round
            self.r.push(r_j);
            if let Some(g) = &self.g {
                Ok(VerifierRoundResult::FinalRound(p0 + r_j * (p1 - p0) == g.evaluate(&self.r).unwrap()))
            } else {
                Err(Error::NoPolySet)
            }
        } else {
            self.r.push(r_j);
            self.expect = p0 + r_j * (p1 - p0);
            Ok(VerifierRoundResult::JthRound(r_j))
        }
    }
}
