use rand::Rng;

use crate::seed::{ConstantSeed, ProgramSeed, TermSeed};

use super::{
    constant::{gen_constant, gen_integer},
    Generator,
};

/// Mutation-based generator: takes seeds from a corpus and applies small mutations.
pub struct Mutator {
    pub version: (usize, usize, usize),
    corpus: Vec<ProgramSeed>,
}

impl Mutator {
    pub fn new(version: (usize, usize, usize)) -> Self {
        Self {
            version,
            corpus: Vec::new(),
        }
    }

    pub fn add_to_corpus(&mut self, seed: ProgramSeed) {
        if self.corpus.len() < 10_000 {
            self.corpus.push(seed);
        } else {
            // Replace a random entry
            let idx = rand::random::<usize>() % self.corpus.len();
            self.corpus[idx] = seed;
        }
    }

    pub fn corpus_size(&self) -> usize {
        self.corpus.len()
    }
}

impl Generator for Mutator {
    fn generate_batch(&self, rng: &mut rand_xoshiro::Xoshiro256PlusPlus, batch_size: usize) -> Vec<ProgramSeed> {
        if self.corpus.is_empty() {
            return Vec::new();
        }

        (0..batch_size)
            .map(|_| {
                let base = &self.corpus[rng.gen_range(0..self.corpus.len())];
                let mut seed = base.clone();
                // Apply 1-3 mutations
                let nmutations = rng.gen_range(1..=3);
                for _ in 0..nmutations {
                    seed.term = mutate_term(rng, &seed.term, 0);
                }
                seed
            })
            .collect()
    }

    fn name(&self) -> &str {
        "mutator"
    }
}

fn mutate_term(rng: &mut impl Rng, term: &TermSeed, depth: usize) -> TermSeed {
    // Probability of mutating at this node vs descending
    let mutate_here = if depth > 8 {
        true
    } else {
        rng.gen_range(0..5) == 0
    };

    if mutate_here {
        return apply_mutation(rng, term);
    }

    // Descend into a random child
    match term {
        TermSeed::Lambda(body) => {
            TermSeed::Lambda(Box::new(mutate_term(rng, body, depth + 1)))
        }
        TermSeed::Apply(fun, arg) => {
            if rng.gen() {
                TermSeed::Apply(Box::new(mutate_term(rng, fun, depth + 1)), arg.clone())
            } else {
                TermSeed::Apply(fun.clone(), Box::new(mutate_term(rng, arg, depth + 1)))
            }
        }
        TermSeed::Delay(body) => {
            TermSeed::Delay(Box::new(mutate_term(rng, body, depth + 1)))
        }
        TermSeed::Force(body) => {
            TermSeed::Force(Box::new(mutate_term(rng, body, depth + 1)))
        }
        TermSeed::Case { constr, branches } => {
            if branches.is_empty() || rng.gen_range(0..3) == 0 {
                TermSeed::Case {
                    constr: Box::new(mutate_term(rng, constr, depth + 1)),
                    branches: branches.clone(),
                }
            } else {
                let idx = rng.gen_range(0..branches.len());
                let mut branches = branches.clone();
                branches[idx] = mutate_term(rng, &branches[idx], depth + 1);
                TermSeed::Case {
                    constr: constr.clone(),
                    branches,
                }
            }
        }
        TermSeed::Constr { tag, fields } => {
            if fields.is_empty() {
                apply_mutation(rng, term)
            } else {
                let idx = rng.gen_range(0..fields.len());
                let mut fields = fields.clone();
                fields[idx] = mutate_term(rng, &fields[idx], depth + 1);
                TermSeed::Constr {
                    tag: *tag,
                    fields,
                }
            }
        }
        // Leaf nodes: always mutate
        _ => apply_mutation(rng, term),
    }
}

fn apply_mutation(rng: &mut impl Rng, term: &TermSeed) -> TermSeed {
    match rng.gen_range(0..12) {
        // Replace with a different constant
        0 => TermSeed::Constant(gen_constant(rng)),
        // Wrap in force
        1 => TermSeed::Force(Box::new(term.clone())),
        // Wrap in delay
        2 => TermSeed::Delay(Box::new(term.clone())),
        // Wrap in lambda
        3 => TermSeed::Lambda(Box::new(term.clone())),
        // Replace with error
        4 => TermSeed::Error,
        // Change a constant value
        5 => match term {
            TermSeed::Constant(ConstantSeed::Integer(i)) => {
                TermSeed::Constant(ConstantSeed::Integer(mutate_integer(rng, *i)))
            }
            TermSeed::Constant(ConstantSeed::ByteString(bs)) => {
                TermSeed::Constant(ConstantSeed::ByteString(mutate_bytestring(rng, bs)))
            }
            TermSeed::Constant(ConstantSeed::Boolean(b)) => {
                TermSeed::Constant(ConstantSeed::Boolean(!b))
            }
            _ => TermSeed::Constant(gen_constant(rng)),
        },
        // Change a var index
        6 => match term {
            TermSeed::Var(idx) => {
                let new_idx = match rng.gen_range(0..4) {
                    0 => idx.saturating_sub(1).max(1),
                    1 => idx + 1,
                    2 => 1,
                    _ => rng.gen_range(1..=10),
                };
                TermSeed::Var(new_idx)
            }
            _ => term.clone(),
        },
        // Change builtin to a different one with same arity
        7 => match term {
            TermSeed::Builtin(id) => {
                let arity = crate::seed::builtin_arity(*id);
                let forces = crate::seed::builtin_force_count(*id);
                // Try to find a different builtin with same arity+forces
                let new_id = (0..=91u8)
                    .filter(|&b| {
                        crate::seed::builtin_arity(b) == arity
                            && crate::seed::builtin_force_count(b) == forces
                            && b != *id
                    })
                    .collect::<Vec<_>>();
                if new_id.is_empty() {
                    TermSeed::Builtin(*id)
                } else {
                    TermSeed::Builtin(new_id[rng.gen_range(0..new_id.len())])
                }
            }
            _ => term.clone(),
        },
        // Apply to a random argument
        8 => {
            let arg = TermSeed::Constant(gen_constant(rng));
            TermSeed::Apply(Box::new(term.clone()), Box::new(arg))
        }
        // Unwrap one layer
        9 => match term {
            TermSeed::Force(inner)
            | TermSeed::Delay(inner)
            | TermSeed::Lambda(inner) => *inner.clone(),
            TermSeed::Apply(fun, _) => *fun.clone(),
            _ => term.clone(),
        },
        // Replace with constr
        10 => TermSeed::Constr {
            tag: rng.gen_range(0..=3),
            fields: vec![term.clone()],
        },
        // Replace with case
        _ => TermSeed::Case {
            constr: Box::new(TermSeed::Constr {
                tag: 0,
                fields: vec![],
            }),
            branches: vec![term.clone()],
        },
    }
}

fn mutate_integer(rng: &mut impl Rng, i: i128) -> i128 {
    match rng.gen_range(0..8) {
        0 => i + 1,
        1 => i - 1,
        2 => -i,
        3 => 0,
        4 => 1,
        5 => i.wrapping_mul(2),
        6 => gen_integer(rng),
        _ => i ^ (1 << rng.gen_range(0..64)),
    }
}

fn mutate_bytestring(rng: &mut impl Rng, bs: &[u8]) -> Vec<u8> {
    let mut result = bs.to_vec();
    if result.is_empty() {
        result.push(rng.gen());
        return result;
    }
    match rng.gen_range(0..5) {
        // Flip a random bit
        0 => {
            let idx = rng.gen_range(0..result.len());
            let bit = rng.gen_range(0..8);
            result[idx] ^= 1 << bit;
        }
        // Change a random byte
        1 => {
            let idx = rng.gen_range(0..result.len());
            result[idx] = rng.gen();
        }
        // Add a byte
        2 => {
            let pos = rng.gen_range(0..=result.len());
            result.insert(pos, rng.gen());
        }
        // Remove a byte
        3 => {
            let idx = rng.gen_range(0..result.len());
            result.remove(idx);
        }
        // Truncate
        _ => {
            if result.len() > 1 {
                result.truncate(rng.gen_range(1..result.len()));
            }
        }
    }
    result
}
