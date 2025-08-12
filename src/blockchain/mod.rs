pub mod block;
pub mod model;

pub use block::Block;
pub use model::Blockchain;

/// Default Proof-of-Work difficulty (number of leading zeros).
pub const DEFAULT_DIFFICULTY: u32 = 3;

/// Base block subsidy (dev value).
pub const BASE_REWARD: u64 = 50;

/// Target seconds per block for auto-adjust
pub const TARGET_BLOCK_TIME_SECS: i64 = 60;

/// How many recent intervals to average when adjusting difficulty
pub const DIFF_ADJUST_WINDOW: usize = 10;

/// Tolerance around the target before we adjust (+/- 20%)
pub const DIFF_ADJUST_THRESHOLD_PCT: f64 = 0.20;

/// Difficulty bounds (keep low in dev to avoid long waits)
pub const DIFF_MIN: u32 = 1;
pub const DIFF_MAX: u32 = 6;
