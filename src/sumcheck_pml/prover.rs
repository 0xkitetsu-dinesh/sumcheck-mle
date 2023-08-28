use ark_ff::Field;
use crate::sumcheck_pml::poly::*;
// The state of the Prover.
pub struct Prover<F: Field, P: SumCheckPolynomial<F>> {
    g: Vec<P>,  // $g$ a list of polynomial being used in this run of the protocol.
    claim: F,   // $claim$ a value prover _claim_ equal the true answer.
    r: Vec<F>,  // Random values $r_1,...,r_j$ sent by the [`Verifier`] in the previous rounds.
    num_vars: usize,
    table:Vec<Vec<F>>   // vector of mle
}

impl<F: Field, P: SumCheckPolynomial<F>> Prover<F, P> {
    /// Create a new [`Prover`] state with the polynomial $g$.
    pub fn new(g: Vec<P>) -> Self {

        let num_vars = g[0].num_vars();
        let mut table = Vec::new();

        // populate table with mle
        for poly in g.iter(){
            assert_eq!(num_vars,poly.num_vars());
            table.push(poly.to_evaluations());
        }

        let mut products = vec![F::one();2usize.pow(num_vars as u32)];

        for poly in table.iter(){
            products = products.iter().zip(poly.iter()).map(|(x, y)| *x * *y).collect::<Vec<F>>();
        }

        Self {
            g,
            claim:products.iter().sum(),
            num_vars,
            r: Vec::with_capacity(num_vars),
            table
        }
    }

    /// Get the value $claim$ that prover claim equal true answer.
    pub fn claim(&self) -> F { self.claim }

    /// Perform $j$-th round of the [`Prover`] side of the prococol.
    pub fn round(&mut self, r_prev: F, round_j: usize) -> Vec<F> {

        let nv = self.num_vars;     // no of variables
        let np = self.table.len();  // poly count
        
        if round_j != 0 {
            self.r.push(r_prev);
            // g.fix_variables(&[r_prev])
            for j in 0..np{
                for b in 0..2usize.pow((nv-round_j) as u32){
                    self.table[j][b] = self.table[j][b << 1] * (F::one() - r_prev) + self.table[j][(b << 1) + 1] * r_prev
                }
            }
        }
        let mut product_sum = vec![F::zero();np+1];

        for b in 0..2usize.pow((nv - round_j - 1) as u32){
            for t in 0..np+1{   // evaluating points
                let mut product = F::one();
                for j in 0..np{
                    let table = &self.table[j];
                    product *= table[b << 1] * (F::one() - F::from(t as u32)) + table[(b << 1) + 1] * F::from(t as u32);
                }
                product_sum[t] += product;
            }
        }

        product_sum
    }

    pub fn num_vars(&self) -> usize {
        self.num_vars
    }
}
