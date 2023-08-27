use ark_ff::Field;
use crate::poly::*;
use polynomial::Polynomial as IPoly;
/// The state of the Verifier.
pub struct Verifier<F: Field, P: SumCheckPolynomial<F>> {
    nv: usize,// Number of variables in the original polynomial.
    claim: F,// A $claim$ value claimed by the Prover.
    r: Vec<F>,// Previously picked random values $r_1,...,r_{j-1}$.
    g: Option<Vec<P>>,// Original polynomial for oracle access
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
    pub fn new(g: Option<Vec<P>>,claim: F) -> Self {
		let num_vars = g.as_ref().unwrap()[0].num_vars();
        Self {
            nv:num_vars,
            claim,
            r: Vec::with_capacity(num_vars),
            g,
            expect:claim
        }
    }

    pub fn univariate_interpolate_and_evaluate(ys:&Vec<F>,eval_at:F) -> F {
        let xs:Vec<F> = (0..ys.len()).map(|x| F::from(x as u32)).collect();
        let poly = IPoly::lagrange(&xs,&ys).unwrap();
        poly.eval(eval_at)
    }

    /// Perform the $j$-th round of the [`Verifier`] side of the protocol.
    pub fn round<R: RngF<F>>(&mut self,p:Vec<F>,rng: &mut R) -> Result<VerifierRoundResult<F>, Error> {
		let r_j = rng.draw();

        if self.expect != p[0] + p[1] {
            Err(Error::ProverClaimMismatch(
                format!("{:?}", self.expect),
                format!("{:?} {:?}", p[0] ,p[1]),
            ))
        } else if self.r.len() == (self.nv - 1) {
            // Last round
            self.r.push(r_j);
            if let Some(g) = &self.g {
                Ok(VerifierRoundResult::FinalRound(Self::univariate_interpolate_and_evaluate(&p,r_j) == 
                                            g.iter().map(|f| f.evaluate(&self.r).unwrap()).product()
                                        ))
            } else {
                Err(Error::NoPolySet)
            }
        } else {
            self.r.push(r_j);
            self.expect = Self::univariate_interpolate_and_evaluate(&p,r_j);
            Ok(VerifierRoundResult::JthRound(r_j))
        }
    }
}
