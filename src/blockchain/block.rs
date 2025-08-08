use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A single block in the blockchain.
/// For now we keep a simple `data` payload; we'll evolve to real transactions later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64, // Unix timestamp (UTC)
    pub previous_hash: String,
    pub nonce: u64,   // Proof-of-Work nonce
    pub hash: String, // Cached hash of the block
    pub data: String, // Placeholder for transactions/payload
}

impl Block {
    /// Create the genesis block (first block in the chain).
    pub fn genesis() -> Self {
        let mut block = Self {
            index: 0,
            timestamp: Utc::now().timestamp(),
            previous_hash: String::from("0"),
            nonce: 0,
            hash: String::new(),
            data: String::from("genesis"),
        };
        block.hash = block.compute_hash();
        block
    }

    /// Create a new block (not mined yet). Call `mine()` to perform PoW.
    pub fn new(index: u64, previous_hash: String, data: String) -> Self {
        let mut block = Self {
            index,
            timestamp: Utc::now().timestamp(),
            previous_hash,
            nonce: 0,
            hash: String::new(),
            data,
        };
        block.hash = block.compute_hash();
        block
    }

    /// Compute the SHA-256 hash of this block using its fields
    /// (excluding the `hash` field itself).
    pub fn compute_hash(&self) -> String {
        let preimage = format!(
            "{}:{}:{}:{}:{}",
            self.index, self.timestamp, self.previous_hash, self.nonce, self.data
        );
        let mut hasher = Sha256::new();
        hasher.update(preimage.as_bytes());
        let digest = hasher.finalize();
        hex::encode(digest)
    }

    /// Perform Proof-of-Work by finding a nonce that yields a hash
    /// starting with `difficulty` leading zeros (in hex).
    pub fn mine(&mut self, difficulty: u32) {
        let target_prefix = "0".repeat(difficulty as usize);
        loop {
            self.hash = self.compute_hash();
            if self.hash.starts_with(&target_prefix) {
                break;
            }
            self.nonce = self.nonce.wrapping_add(1);
        }
    }

    /// Validate that the block's cached `hash` matches its content and
    /// satisfies the PoW difficulty. (Does NOT validate chain linkage.)
    pub fn is_valid(&self, difficulty: u32) -> bool {
        let expected = self.compute_hash();
        if self.hash != expected {
            return false;
        }
        self.hash
            .chars()
            .take(difficulty as usize)
            .all(|c| c == '0')
    }
}

#[cfg(test)]
mod tests {
    use super::Block;

    #[test]
    fn genesis_has_valid_hash() {
        let b = Block::genesis();
        assert_eq!(b.hash, b.compute_hash());
        assert!(!b.hash.is_empty());
    }

    #[test]
    fn mining_produces_leading_zeros() {
        let mut b = Block::new(1, "prev".into(), "hello".into());
        b.mine(2);
        assert!(b.hash.starts_with("00"));
        assert!(b.is_valid(2));
    }

    #[test]
    fn invalid_when_mutated() {
        let mut b = Block::new(2, "prev".into(), "payload".into());
        b.mine(2);
        let old_hash = b.hash.clone();
        b.data = "tampered".into();
        assert_ne!(old_hash, b.compute_hash());
        assert!(!b.is_valid(2));
    }
}
