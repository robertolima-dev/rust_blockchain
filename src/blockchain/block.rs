use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::transaction::Transaction;

/// A single block in the blockchain holding a list of transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64, // Unix timestamp (UTC)
    pub previous_hash: String,
    pub nonce: u64,   // Proof-of-Work nonce
    pub hash: String, // Cached hash of the block
    pub transactions: Vec<Transaction>,
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
            transactions: Vec::new(), // we can later include a coinbase if we want
        };
        block.hash = block.compute_hash();
        block
    }

    /// Create a new block (not mined yet). Call `mine()` to perform PoW.
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let mut block = Self {
            index,
            timestamp: Utc::now().timestamp(),
            previous_hash,
            nonce: 0,
            hash: String::new(),
            transactions,
        };
        block.hash = block.compute_hash();
        block
    }

    /// Compute the SHA-256 hash of this block using its fields
    /// (excluding the `hash` field itself). Transactions are serialized
    /// deterministically as JSON and included in the preimage.
    pub fn compute_hash(&self) -> String {
        let txs_json = serde_json::to_string(&self.transactions).expect("serialize txs");
        let preimage = format!(
            "{}:{}:{}:{}:{}",
            self.index, self.timestamp, self.previous_hash, self.nonce, txs_json
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
    use crate::transaction::{OutPoint, Transaction, TxInput, TxOutput};

    #[test]
    fn genesis_has_valid_hash() {
        let b = Block::genesis();
        assert_eq!(b.hash, b.compute_hash());
        assert!(!b.hash.is_empty());
    }

    #[test]
    fn mining_produces_leading_zeros() {
        let tx = Transaction::new(
            vec![TxInput {
                outpoint: OutPoint {
                    txid: "demo-txid".into(),
                    vout: 0,
                },
            }],
            vec![TxOutput {
                address: "addr".into(),
                amount: 1,
            }],
        );
        let mut b = Block::new(1, "prev".into(), vec![tx]);
        b.mine(2);
        assert!(b.hash.starts_with("00"));
        assert!(b.is_valid(2));
    }

    #[test]
    fn invalid_when_mutated() {
        let tx = Transaction::new(
            vec![TxInput {
                outpoint: OutPoint {
                    txid: "demo-txid".into(),
                    vout: 0,
                },
            }],
            vec![TxOutput {
                address: "addr".into(),
                amount: 1,
            }],
        );
        let mut b = Block::new(2, "prev".into(), vec![tx]);
        b.mine(2);
        let old_hash = b.hash.clone();

        // Mutate: add a new tx (tampering)
        let extra = Transaction::new(
            vec![TxInput {
                outpoint: OutPoint {
                    txid: "x".into(),
                    vout: 0,
                },
            }],
            vec![TxOutput {
                address: "y".into(),
                amount: 1,
            }],
        );
        b.transactions.push(extra);

        assert_ne!(old_hash, b.compute_hash());
        assert!(!b.is_valid(2));
    }
}
