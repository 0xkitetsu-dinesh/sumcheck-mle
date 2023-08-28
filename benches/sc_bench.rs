#[macro_use]
extern crate criterion;
use criterion::{black_box, BenchmarkId, Criterion};
use sumcheck_mle::{sort_arr, sumcheck_pml,sumcheck_naive};
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


#[derive(MontConfig)]
#[modulus = "97"]
#[generator = "5"]
struct FrConfig;
type Fp97 = Fp64<MontBackend<FrConfig, 1>>;


fn rand_poly<R: Rng,F:Field>(l: usize, d: usize, rng: &mut R) -> SparsePolynomial<F, SparseTerm> {
    let mut random_terms = Vec::new();
    let num_terms = rng.gen_range(1..1000);
    // For each term, randomly select up to `l` variables with degree
    // in [1,d] and random coefficient
    random_terms.push((F::rand(rng), SparseTerm::new(vec![])));
    for _ in 1..num_terms {
        let term = (0..l)
            .map(|i| {
                if rng.gen_bool(0.5) {
                    Some((i, rng.gen_range(1..(d + 1))))
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        let coeff = F::rand(rng);
        random_terms.push((coeff, SparseTerm::new(term)));
    }
    SparsePolynomial::from_coefficients_slice(l, &random_terms)
}
/// Perform a naive n^2 multiplication of `self` by `other`.
fn naive_mul<F:Field>(
    cur: &SparsePolynomial<F, SparseTerm>,
    other: &SparsePolynomial<F, SparseTerm>,
) -> SparsePolynomial<F, SparseTerm> {
    if cur.is_zero() || other.is_zero() {
        SparsePolynomial::zero()
    } else {
        let mut result_terms = Vec::new();
        for (cur_coeff, cur_term) in cur.terms().into_iter() {
            for (other_coeff, other_term) in other.terms().iter() {
                let mut term:Vec<(usize, usize)> = cur_term.to_vec();

                // term.extend(other_term.0.clone());
                term.extend(&other_term.to_vec());

                // println!("{} , {:?} || {} , {:?} || {:?} , {:?}",cur_coeff,cur_term,other_coeff,other_term,&term);
                // println!("cur_term.0 => {:?} other_term.0 => {:?}",cur_term.iter().collect::<Vec<_>>(),other_term.iter().collect::<Vec<_>>());

                result_terms.push((*cur_coeff * *other_coeff, SparseTerm::new(term)));

            }
        }
        SparsePolynomial::from_coefficients_slice(cur.num_vars, result_terms.as_slice())
    }
}

fn prove_sumcheck_pml(c:&mut Criterion){
    let rng = &mut test_rng();
    let mut p = Vec::new();
    p.push(rand_poly::<_, Fp97>(2, 1, rng));
    p.push(rand_poly::<_, Fp97>(2, 1, rng));

    let mut product = naive_mul(&p[0], &p[1]);

    c.bench_function("prove sumcheck pml", |b| b.iter(|| sumcheck_pml::prove_bench(black_box(p.clone()))));
    c.bench_function("prove sumcheck naive", |b| b.iter(|| sumcheck_naive::prove_bench(black_box(product.clone()))));
}

fn sort_bench(c:&mut Criterion){
    let mut arr = black_box([6,2,4,1,-9,5]);

    c.bench_function("sorting algo", |b| b.iter(|| sort_arr(&mut arr)));
}
criterion_group!(benches,prove_sumcheck_pml);
criterion_main!(benches);