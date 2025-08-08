pub mod block;
pub mod model;

pub use block::Block;
pub use model::Blockchain;

/// Default Proof-of-Work difficulty (number of leading zeros).
/// Keep this low during development to avoid slow mining.
pub const DEFAULT_DIFFICULTY: u32 = 3;
