pub mod prover;
pub mod verifier;
pub mod poly;

#[cfg(test)]
mod test;

use ark_std::{rand::Rng, test_rng};
use ark_ff::{Field, Zero};
use ark_poly::{
    multivariate::{self,SparsePolynomial,SparseTerm, Term},
    DenseMVPolynomial,
};
use ark_ff::{
	fields::Fp64,
	fields::{MontBackend, MontConfig},
	One, PrimeField,
};


use crate::sumcheck_pml::prover::*;
use crate::sumcheck_pml::verifier::*;
use crate::sumcheck_pml::poly::*;

#[derive(MontConfig)]
#[modulus = "97"]
#[generator = "5"]
struct FrConfig;
type Fp97 = Fp64<MontBackend<FrConfig, 1>>;

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

pub fn prove_bench<F: Field, P: SumCheckPolynomial<F>>(g:Vec<P>){
    let mut prover = Prover::new(g);
    // let mut verifier = Verifier::new(Some(g),prover.claim());
    
    let rng = &mut test_rng();
    let mut r_j = F::one();

    for j in 0..prover.num_vars() {
        let p = prover.round(r_j, j);
        r_j = rng.draw();
    }
}
