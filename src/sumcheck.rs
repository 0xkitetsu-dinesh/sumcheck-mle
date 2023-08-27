#[allow(dead_code)]
use std::marker::PhantomData;
use ark_ff::Zero;
use ark_ff::{
	fields::Fp64,
	fields::{MontBackend, MontConfig},
	Field, One, PrimeField,
};
use ark_poly::{
    multivariate::{self, SparseTerm, Term},
    DenseMVPolynomial,
    univariate, Polynomial,
};
use ark_std::{rand::Rng, test_rng};
use bitvec::slice::BitSlice;

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

    /// Reduce the number of variables in `Self` by fixing a
    /// `partial_point.len()` variables at `partial_point`.
    fn fix_variables(&self, partial_point: &[F]) -> Self;

    fn to_univariate(&self) -> univariate::SparsePolynomial<F>;

    /// Returns the number of variables in `self`
    fn num_vars(&self) -> usize;

    /// Returns a list of evaluations over the entire BooleanHypercube domain
    fn to_evaluations(&self) -> Vec<F>;
}

impl<F: Field> SumCheckPolynomial<F> for multivariate::SparsePolynomial<F, SparseTerm> {
    fn evaluate(&self, point: &[F]) -> Option<F> {
        Some(Polynomial::evaluate(self, &point.to_owned()))
    }

    fn fix_variables(&self, partial_point: &[F]) -> Self {
		// println!("{:?}",&partial_point);
        let mut res = Self::zero();
        let num_vars = DenseMVPolynomial::num_vars(self);
        let mut full_point = partial_point.to_vec();
        full_point.append(&mut vec![F::one(); num_vars - partial_point.len()]);

        for (coeff, term) in self.terms() {
            let mut eval = term.evaluate(&full_point);
            eval *= coeff;
            let new_term = SparseTerm::new(
                term.iter()
                    .filter(|(var, _)| *var >= partial_point.len())
                    .map(|(var, power)| (var - partial_point.len(), *power))
                    .collect(),
            );
            let poly = multivariate::SparsePolynomial {
                num_vars: num_vars - partial_point.len(),
                terms: vec![(eval, new_term)],
            };

            res += &poly;
        }

        res
    }

    fn to_univariate(&self) -> univariate::SparsePolynomial<F> {
        let mut res = univariate::SparsePolynomial::zero();
		// println!("Prover::to_uni()# Boolean Hyper cube {:?} ",(DenseMVPolynomial::num_vars(self) - 1) as u32);

        for p in BooleanHypercube::new((DenseMVPolynomial::num_vars(self) - 1) as u32) {
            let mut point = vec![F::one()];
			// println!("Prover::to_uni()# p {:?}",&p);
			// println!("Prover::to_uni()# b4 {:?}",&point);
            point.extend_from_slice(&p);
			// println!("Prover::to_uni()# a4 {:?}",&point);
            let mut r = univariate::SparsePolynomial::zero();

            for (coeff, term) in self.terms() {
				// println!("----");
				// println!("Prover::to_uni()# I coeff,term {:?} {:?}",&coeff,&term);
                let mut eval = term.evaluate(&point);
				// println!("Prover::to_uni()# II eval {:?}",&eval);
				
				
                let power = term
                    .iter()
                    .find(|(variable, _power)| *variable == 0)
                    .map(|(_variable, power)| *power)
                    .unwrap_or(0);

                eval *= coeff;

				// println!("Prover::to_uni()# III power eval term {:?} {:?} {:?}",&power,&eval,&term);
				
				
				
                r += &univariate::SparsePolynomial::from_coefficients_slice(&[(power, eval)]);
				// println!("Prover::to_uni()# III r {:?}",&r);
            }

            res += &r;
        }
		// println!("Prover::to_uni()# end {:?}",&res);
		// todo!();
        res
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

/// The state of the Prover.
pub struct Prover<F: Field, P: SumCheckPolynomial<F>> {
    /// $g$ a polynomial being used in this run of the protocol.
    g: P,

    /// $C_1$ a value prover _claims_ equal the true answer.
    c_1: F,

    /// Random values $r_1,...,r_j$ sent by the [`Verifier`] in the
    /// previous rounds.
    r: Vec<F>,
    num_vars: usize,
}

impl<F: Field, P: SumCheckPolynomial<F>> Prover<F, P> {
    /// Create a new [`Prover`] state with the polynomial $g$.
    pub fn new(g: P) -> Self {
        let c_1 = g.to_evaluations().into_iter().sum();
        let num_vars = g.num_vars();
        Self {
            g,
            c_1,
            num_vars,
            r: Vec::with_capacity(num_vars),
        }
    }

    /// Get the value $C_1$ that prover claims equal true answer.
    pub fn c_1(&self) -> F {
        self.c_1
    }

    /// Perform $j$-th round of the [`Prover`] side of the prococol.
    pub fn round(&mut self, r_prev: F, j: usize) -> univariate::SparsePolynomial<F> {
		// println!("Prover::round()# r_prev,j {:?} {:?}",&r_prev,&j);
        if j != 0 {
            self.r.push(r_prev);
            self.g = self.g.fix_variables(&[r_prev]);
        }

		// println!("------------------------------------------");
		// println!("Prover::round()# self.g.to_univariate() {:?} ",&self.g.to_univariate());
		// println!("------------------------------------------");

        self.g.to_univariate()
    }

    pub fn num_vars(&self) -> usize {
        self.num_vars
    }
}

/// The state of the Verifier.
pub struct Verifier<F: Field, P: SumCheckPolynomial<F>> {
    /// Number of variables in the original polynomial.
    n: usize,

    /// A $C_1$ value claimed by the Prover.
    c_1: F,

    /// Univariate polynomials $g_1,...,g_{j-1}$ received from the [`Prover`].
    g_part: Vec<univariate::SparsePolynomial<F>>,

    /// Previously picked random values $r_1,...,r_{j-1}$.
    r: Vec<F>,

    /// Original polynomial for oracle access
    g: Option<P>,
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
    /// $C_1$ - the value claimed to be true answer by the [`Prover`].
    /// $g$ - the polynimial itself for oracle access by the [`Verifier`].
    pub fn new(g: Option<P>) -> Self {
		let num_vars = g.as_ref().unwrap().num_vars();
        Self {
            n:num_vars,
            c_1: F::zero(),
            g_part: Vec::with_capacity(num_vars),
            r: Vec::with_capacity(num_vars),
            g,
        }
    }

    pub fn set_c_1(&mut self, c_1: F) {
        self.c_1 = c_1;
    }

    /// Perform the $j$-th round of the [`Verifier`] side of the protocol.
    ///
    /// $g_j$ - a univariate polynomial sent in this round by the [`Prover`].
    pub fn round<R: RngF<F>>(
        &mut self,
        g_j: univariate::SparsePolynomial<F>,
        rng: &mut R,
    ) -> Result<VerifierRoundResult<F>, Error> {
		let r_j = rng.draw();
		println!("V::round()# j g_j {:?} {:?} {:?}",self.r,&r_j,g_j);
		println!("---");
        if self.r.is_empty() {
            // First Round
            let evaluation = g_j.evaluate(&F::zero()) + g_j.evaluate(&F::one());
			println!("V::round()# 1st round eval,c {:?} {:?}",evaluation,self.c_1);
            if self.c_1 != evaluation {
                Err(Error::ProverClaimMismatch(
                    format!("start {:?}", self.c_1),
                    format!("{:?}", evaluation),
                ))
            } else {
                self.g_part.push(g_j);
                self.r.push(r_j);

                Ok(VerifierRoundResult::JthRound(r_j))
            }
        } else if self.r.len() == (self.n - 1) {
            // Last round
            self.r.push(r_j);

            if let Some(g) = &self.g {
                assert_eq!(g_j.evaluate(&r_j), g.evaluate(&self.r).unwrap());
				// println!("V::round()# last g_j,g {:?} {:?}",g_j.evaluate(&r_j),g.evaluate(&self.r).unwrap());

                Ok(VerifierRoundResult::FinalRound(
                    g_j.evaluate(&r_j) == g.evaluate(&self.r).unwrap(),
                ))
            } else {
                Err(Error::NoPolySet)
            }
        } else {
            // j-th round
            let g_jprev = self.g_part.last().unwrap();
            let r_jprev = self.r.last().unwrap();

            let prev_evaluation = g_jprev.evaluate(r_jprev);
            let evaluation = g_j.evaluate(&F::zero()) + g_j.evaluate(&F::one());
			// println!("V::round()# j rprev_evaluation,evaluation {:?} {:?}",prev_evaluation,evaluation);
            if prev_evaluation != evaluation {
                return Err(Error::ProverClaimMismatch(
                    format!("{:?}", prev_evaluation),
                    format!("{:?}", evaluation),
                ));
            }
			
            self.g_part.push(g_j);
            self.r.push(r_j);

            Ok(VerifierRoundResult::JthRound(r_j))
        }
    }
}

fn main(){
	// #[derive(MontConfig)]
    // #[modulus = "71"]
    // #[generator = "7"]
    // struct FrConfig;

	// https://cronokirby.com/notes/2022/09/the-goldilocks-field/
	// #[derive(MontConfig)]
    // #[modulus = "18446744069414584321"]
    // #[generator = "7"]
    // struct FrConfig;

    #[derive(MontConfig)]
    #[modulus = "97"]
    #[generator = "5"]
    struct FrConfig;


    type GL64 = Fp64<MontBackend<FrConfig, 1>>;
    type Fp97 = Fp64<MontBackend<FrConfig, 1>>;


	let rng = &mut test_rng();
        // 2 *x_0^3 + x_0 * x_2 + x_1 * x_2
	// let g: multivariate::SparsePolynomial<_, SparseTerm> = multivariate::SparsePolynomial::from_coefficients_slice(
	// 	3,
	// 	&[
	// 		(
	// 			GL64::from_bigint(2u32.into()).unwrap(),
	// 			multivariate::SparseTerm::new(vec![(0, 3)]),
	// 		),
	// 		(
	// 			GL64::from_bigint(1u32.into()).unwrap(),
	// 			multivariate::SparseTerm::new(vec![(0, 1), (2, 1)]),
	// 		),
	// 		(
	// 			GL64::from_bigint(1u32.into()).unwrap(),
	// 			multivariate::SparseTerm::new(vec![(1, 1), (2, 1)]),
	// 		),
	// 	],
	// );
    let g: multivariate::SparsePolynomial<_, SparseTerm> = multivariate::SparsePolynomial::from_coefficients_slice(
		2,
		&[
			(
				Fp97::from_bigint(20u32.into()).unwrap(),
				multivariate::SparseTerm::new(vec![(0, 2)]),
			),
			(
				Fp97::from_bigint(5u32.into()).unwrap(),
				multivariate::SparseTerm::new(vec![(0, 2), (1, 1)]),
			),
			(
				Fp97::from_bigint(29u32.into()).unwrap(),
				multivariate::SparseTerm::new(vec![(0, 1), (1, 1)]),
			),
            (
				Fp97::from_bigint(62u32.into()).unwrap(),
				multivariate::SparseTerm::new(vec![(0, 2), (1, 2)]),
			),
            (
				Fp97::from_bigint(90u32.into()).unwrap(),
				multivariate::SparseTerm::new(vec![(0, 1), (1, 2)]),
			),
            (
				Fp97::from_bigint(88u32.into()).unwrap(),
				multivariate::SparseTerm::new(vec![(1, 2)]),
			),
		],
	);
    // // 24 * x_0   +   15 * x_0 * x_1   +   35 * x_1
    // let g: multivariate::SparsePolynomial<_, SparseTerm> = multivariate::SparsePolynomial::from_coefficients_slice(
	// 	2,
	// 	&[
	// 		(
	// 			Fp97::from_bigint(24u32.into()).unwrap(),
	// 			multivariate::SparseTerm::new(vec![(0, 1)]),
	// 		),
	// 		(
	// 			Fp97::from_bigint(15u32.into()).unwrap(),
	// 			multivariate::SparseTerm::new(vec![(0, 1), (1, 1)]),
	// 		),
	// 		(
	// 			Fp97::from_bigint(35u32.into()).unwrap(),
	// 			multivariate::SparseTerm::new(vec![(1, 1)]),
	// 		),
	// 	],
	// );
	println!("=============== Given Polynomial ===================");
	println!("{:?}",g.clone());
	println!("==========================================================");
	let mut prover = Prover::new(g.clone());
	let c_1 = prover.c_1();
	println!("claim H => {:?}",c_1);
	let mut r_j = Fp97::one();
	let mut verifier = Verifier::new(Some(g));
	verifier.set_c_1(c_1);

	for j in 0..3 {
		let g_j = prover.round(r_j, j);
		let verifier_res = verifier.round(g_j, rng).unwrap();
		match verifier_res {
			VerifierRoundResult::JthRound(r) => {
                println!("match :: round r {:?} {:?} ", j,r);
				r_j = r;
			}
			VerifierRoundResult::FinalRound(res) => {
				assert!(res);
				break;
			}
		}
	}
}