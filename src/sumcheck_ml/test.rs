use ark_ff::{fields::Fp64,fields::{MontBackend, MontConfig},One, PrimeField,};
use ark_poly::{multivariate::{self, Term, SparseTerm},DenseMVPolynomial, };
use ark_std::test_rng;

use crate::sumcheck_ml::prover::*;
use crate::sumcheck_ml::verifier::*;


#[test]
fn test_normal_poly(){
    #[derive(MontConfig)]
    #[modulus = "97"]
    #[generator = "5"]
    struct FrConfig;

    type Fp97 = Fp64<MontBackend<FrConfig, 1>>;


	let rng = &mut test_rng();
        
    // 24 * x_0   +   15 * x_0 * x_1   +   35 * x_1
    let g: multivariate::SparsePolynomial<_, SparseTerm> = multivariate::SparsePolynomial::from_coefficients_slice(
		2,
		&[
			(
				Fp97::from_bigint(24u32.into()).unwrap(),
				multivariate::SparseTerm::new(vec![(0, 1)]),
			),
			(
				Fp97::from_bigint(15u32.into()).unwrap(),
				multivariate::SparseTerm::new(vec![(0, 1), (1, 1)]),
			),
			(
				Fp97::from_bigint(35u32.into()).unwrap(),
				multivariate::SparseTerm::new(vec![(1, 1)]),
			),
		],
	);
	println!("=============== Given Polynomial ===================");
	println!("{:?}",g.clone());
	println!("==========================================================");

    let mut prover = Prover::new(g.clone());
    let mut verifier = Verifier::new(Some(g.clone()),prover.claim());

	println!("prover.claim => {:?}",prover.claim());

    let mut r_j = Fp97::one();

    for j in 0..DenseMVPolynomial::num_vars(&g) {
		let (p0,p1) = prover.round(r_j, j);
		println!("Round-{} p => {:?} , {:?}",j,&p0,&p1);
		let verifier_res = verifier.round(p0,p1, rng).unwrap();
		match verifier_res {
			VerifierRoundResult::JthRound(r) => {
                println!("Round-{} V's r => {:?} expect => {:?}",j,&r,verifier.expect);
				r_j = r;
			}
			VerifierRoundResult::FinalRound(res) => {
				assert!(res);
				break;
			}
		}
	}

}