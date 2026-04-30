pub mod builtin_aware;
pub mod constant;
pub mod mutate;
pub mod random;

use rand_xoshiro::Xoshiro256PlusPlus;

use crate::seed::ProgramSeed;

/// A generator produces batches of program seeds for fuzzing.
/// Uses concrete RNG type for dyn-compatibility and speed.
pub trait Generator: Send + Sync {
    /// Generate a batch of program seeds.
    fn generate_batch(&self, rng: &mut Xoshiro256PlusPlus, batch_size: usize) -> Vec<ProgramSeed>;

    /// Name for statistics/logging.
    fn name(&self) -> &str;
}
