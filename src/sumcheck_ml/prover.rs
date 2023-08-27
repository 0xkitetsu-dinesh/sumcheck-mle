use ark_ff::Field;
use crate::poly::*;
/// The state of the Prover.
pub struct Prover<F: Field, P: SumCheckPolynomial<F>> {
    g: P,   // $g$ a polynomial being used in this run of the protocol.
    claim: F, // $claim$ a value prover _claims_ equal the true answer.
    r: Vec<F>,// Random values $r_1,...,r_j$ sent by the [`Verifier`] in the previous rounds.
    num_vars: usize,
    table:Vec<F>
}

impl<F: Field, P: SumCheckPolynomial<F>> Prover<F, P> {
    /// Create a new [`Prover`] state with the polynomial $g$.
    pub fn new(g: P) -> Self {
        let num_vars = g.num_vars();
        let table = g.to_evaluations();
        let claim = table.iter().sum();

        Self {
            g,
            claim,
            num_vars,
            r: Vec::with_capacity(num_vars),
            table
        }
    }

    /// Get the value $claim$ that prover claims equal true answer.
    pub fn claim(&self) -> F { self.claim }

    /// Perform $j$-th round of the [`Prover`] side of the prococol.
    pub fn round(&mut self, r_prev: F, j: usize) -> (F,F) {
        
        if j != 0 {
            self.r.push(r_prev);
            // g.fix_variables(&[r_prev])
            for b in 0..2usize.pow((self.num_vars - j)as u32){
                self.table[b] = self.table[b<<1] * (F::one() - r_prev) + self.table[(b << 1) + 1] * r_prev;
            }

        }

        let mut p0 = F::zero();
        let mut p1 = F::zero();
        // evaluating points - p0,p1
        for b in 0..2usize.pow((self.num_vars - j - 1)as u32){
            p0 += self.table[b<<1];
            p1 += self.table[(b << 1) + 1];
        }
        
        (p0,p1)
    }

    pub fn num_vars(&self) -> usize {
        self.num_vars
    }
}
