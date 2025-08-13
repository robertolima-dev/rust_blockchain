use super::{
    Block, DIFF_ADJUST_THRESHOLD_PCT, DIFF_ADJUST_WINDOW, DIFF_MAX, DIFF_MIN,
    TARGET_BLOCK_TIME_SECS,
};
use crate::transaction::Transaction;
use log::debug;

/// Simple in-memory blockchain with Proof-of-Work.
#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u32,
}

impl Blockchain {
    /// Initialize a new blockchain with a genesis block.
    pub fn new(difficulty: u32) -> Self {
        let mut bc = Self {
            chain: Vec::new(),
            difficulty,
        };
        bc.chain.push(Block::genesis());
        bc
    }

    /// Return the last block in the chain.
    pub fn last_block(&self) -> &Block {
        self.chain
            .last()
            .expect("Blockchain should always have at least the genesis block")
    }

    /// Mine and append a new block with the provided transactions.
    /// After appending, maybe adjust difficulty for *future* blocks.
    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> &Block {
        let index = self.chain.len() as u64;
        let prev_hash = self.last_block().hash.clone();

        let mut block = Block::new(index, prev_hash, transactions);
        block.mine(self.difficulty);
        self.chain.push(block);

        // Adjust difficulty for the next block (does not affect the one just mined).
        self.maybe_adjust_difficulty();

        self.last_block()
    }

    /// Validate the entire chain: linkage, hashes and PoW.
    pub fn is_valid_chain(&self) -> bool {
        if self.chain.is_empty() {
            return false;
        }

        // Validate genesis block immutability
        let genesis = &self.chain[0];
        if genesis.index != 0
            || genesis.previous_hash != "0"
            || genesis.hash != genesis.compute_hash()
        {
            return false;
        }

        // Validate the rest of the chain
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let prev = &self.chain[i - 1];

            // Check linkage
            if current.previous_hash != prev.hash {
                return false;
            }

            // Check hash integrity + difficulty
            // Note: we validate with current difficulty here; in a real chain you'd
            // store difficulty per block. For our didactic chain, it's acceptable.
            if !current.is_valid(self.difficulty) {
                return false;
            }
        }

        true
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn difficulty(&self) -> u32 {
        self.difficulty
    }

    pub fn set_difficulty(&mut self, difficulty: u32) {
        self.difficulty = difficulty;
    }

    /// Adjust difficulty towards the target block time using the average of the last N intervals.
    /// If average < (1 - tol) * target => increase difficulty by 1 (up to DIFF_MAX)
    /// If average > (1 + tol) * target => decrease difficulty by 1 (down to DIFF_MIN)
    fn maybe_adjust_difficulty(&mut self) {
        // Need at least (window + 1) blocks to get `window` intervals
        if self.chain.len() < DIFF_ADJUST_WINDOW + 1 {
            return;
        }

        // Compute average interval (seconds) over the last `window` gaps
        let start = self.chain.len() - (DIFF_ADJUST_WINDOW + 1);
        let mut total: i64 = 0;
        for i in (start + 1)..(start + 1 + DIFF_ADJUST_WINDOW) {
            let newer = &self.chain[i];
            let older = &self.chain[i - 1];
            let dt = newer.timestamp - older.timestamp;
            // guard against clock anomalies; clamp to at least 1s
            total += dt.max(1);
        }
        let avg_secs = total as f64 / DIFF_ADJUST_WINDOW as f64;

        let target = TARGET_BLOCK_TIME_SECS as f64;
        let lower = target * (1.0 - DIFF_ADJUST_THRESHOLD_PCT);
        let upper = target * (1.0 + DIFF_ADJUST_THRESHOLD_PCT);

        let old = self.difficulty;
        if avg_secs < lower && self.difficulty < DIFF_MAX {
            self.difficulty += 1;
            debug!(
                "Difficulty ↑ {} -> {} (avg {:.1}s < {:.1}s target; window={})",
                old, self.difficulty, avg_secs, target, DIFF_ADJUST_WINDOW
            );
        } else if avg_secs > upper && self.difficulty > DIFF_MIN {
            self.difficulty -= 1;
            debug!(
                "Difficulty ↓ {} -> {} (avg {:.1}s > {:.1}s target; window={})",
                old, self.difficulty, avg_secs, target, DIFF_ADJUST_WINDOW
            );
        } else {
            debug!(
                "Difficulty stays at {} (avg {:.1}s ~ target {:.1}s; window={})",
                self.difficulty, avg_secs, target, DIFF_ADJUST_WINDOW
            );
        }
    }

    /// Append a pre-mined block (nonce/hash already set) after validating linkage and PoW.
    pub fn append_premined_block(&mut self, block: Block) -> Result<(), &'static str> {
        // linkage
        if block.previous_hash != self.last_block().hash {
            return Err("stale template: previous_hash mismatch");
        }
        // PoW at current difficulty (simplificação didática)
        if !block.is_valid(self.difficulty) {
            return Err("invalid PoW for current difficulty");
        }
        self.chain.push(block);
        // adjust difficulty for next blocks
        self.maybe_adjust_difficulty();
        Ok(())
    }
}
