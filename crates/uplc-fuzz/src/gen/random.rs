use rand::Rng;

use crate::seed::{ProgramSeed, TermSeed};

use super::{constant::gen_constant, Generator};

/// Pure random structural generation.
/// Fast, high volume. Most programs will error but that's fine for coverage.
pub struct RandomStructural {
    pub max_depth: usize,
    pub version: (usize, usize, usize),
}

impl Default for RandomStructural {
    fn default() -> Self {
        Self {
            max_depth: 12,
            version: (1, 1, 0),
        }
    }
}

impl Generator for RandomStructural {
    fn generate_batch(&self, rng: &mut rand_xoshiro::Xoshiro256PlusPlus, batch_size: usize) -> Vec<ProgramSeed> {
        (0..batch_size)
            .map(|_| ProgramSeed {
                version: self.version,
                term: gen_term(rng, self.max_depth, 0),
            })
            .collect()
    }

    fn name(&self) -> &str {
        "random"
    }
}

/// Generate a random term. `lambda_depth` tracks how many lambdas we're nested in
/// (for valid DeBruijn index generation).
fn gen_term(rng: &mut impl Rng, max_depth: usize, lambda_depth: usize) -> TermSeed {
    if max_depth == 0 {
        return gen_leaf(rng, lambda_depth);
    }

    // Weighted choice biased toward smaller programs and useful constructs
    match rng.gen_range(0..100) {
        // Leaf nodes (40%)
        0..=9 => gen_constant_term(rng),
        10..=17 => gen_builtin_term(rng),
        18..=22 => {
            if lambda_depth > 0 {
                TermSeed::Var(rng.gen_range(1..=lambda_depth))
            } else {
                gen_constant_term(rng)
            }
        }
        23..=25 => TermSeed::Error,
        // Compound nodes (60%)
        26..=45 => {
            // Apply - the most important combinator
            let fun = gen_term(rng, max_depth - 1, lambda_depth);
            let arg = gen_term(rng, max_depth - 1, lambda_depth);
            TermSeed::Apply(Box::new(fun), Box::new(arg))
        }
        46..=60 => {
            // Lambda
            let body = gen_term(rng, max_depth - 1, lambda_depth + 1);
            TermSeed::Lambda(Box::new(body))
        }
        61..=70 => {
            // Force
            let inner = gen_term(rng, max_depth - 1, lambda_depth);
            TermSeed::Force(Box::new(inner))
        }
        71..=80 => {
            // Delay
            let inner = gen_term(rng, max_depth - 1, lambda_depth);
            TermSeed::Delay(Box::new(inner))
        }
        81..=90 => {
            // Constr
            let tag = rng.gen_range(0..=5);
            let nfields = rng.gen_range(0..=3);
            let fields = (0..nfields)
                .map(|_| gen_term(rng, max_depth - 1, lambda_depth))
                .collect();
            TermSeed::Constr { tag, fields }
        }
        _ => {
            // Case
            let constr = gen_term(rng, max_depth - 1, lambda_depth);
            let nbranches = rng.gen_range(1..=4);
            let branches = (0..nbranches)
                .map(|_| gen_term(rng, max_depth - 1, lambda_depth))
                .collect();
            TermSeed::Case {
                constr: Box::new(constr),
                branches,
            }
        }
    }
}

fn gen_leaf(rng: &mut impl Rng, lambda_depth: usize) -> TermSeed {
    match rng.gen_range(0..10) {
        0..=3 => gen_constant_term(rng),
        4..=6 => gen_builtin_term(rng),
        7..=8 => {
            if lambda_depth > 0 {
                TermSeed::Var(rng.gen_range(1..=lambda_depth))
            } else {
                gen_constant_term(rng)
            }
        }
        _ => TermSeed::Error,
    }
}

fn gen_constant_term(rng: &mut impl Rng) -> TermSeed {
    TermSeed::Constant(gen_constant(rng))
}

fn gen_builtin_term(rng: &mut impl Rng) -> TermSeed {
    // Stick to well-known builtins (0..=91), skip ledger-specific ones
    TermSeed::Builtin(rng.gen_range(0..=91))
}
