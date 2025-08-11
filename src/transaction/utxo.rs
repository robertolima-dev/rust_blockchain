use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::model::{Transaction, TxOutput};

/// Identifies a specific transaction output by its txid and index.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct OutPoint {
    pub txid: String,
    pub vout: u32,
}

impl PartialEq for OutPoint {
    fn eq(&self, other: &Self) -> bool {
        self.txid == other.txid && self.vout == other.vout
    }
}

impl Hash for OutPoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.txid.hash(state);
        self.vout.hash(state);
    }
}

/// A simple UTXO set wrapper over a HashMap.
/// Stores spendable outputs keyed by (txid, vout).
#[derive(Debug, Default)]
pub struct UtxoSet {
    map: HashMap<OutPoint, TxOutput>,
}

impl UtxoSet {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert a single output into the set.
    pub fn insert(&mut self, outpoint: OutPoint, output: TxOutput) {
        self.map.insert(outpoint, output);
    }

    /// Spend (remove) a single outpoint. Returns the removed output if it existed.
    pub fn spend(&mut self, outpoint: &OutPoint) -> Option<TxOutput> {
        self.map.remove(outpoint)
    }

    pub fn get(&self, outpoint: &OutPoint) -> Option<&TxOutput> {
        self.map.get(outpoint)
    }

    pub fn contains(&self, outpoint: &OutPoint) -> bool {
        self.map.contains_key(outpoint)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Read-only iterator over all entries (for debugging/observability).
    pub fn iter(&self) -> impl Iterator<Item = (&OutPoint, &TxOutput)> {
        self.map.iter()
    }

    /// Utility to add all outputs of a tx (used when applying a mined block).
    pub fn add_tx_outputs(&mut self, tx: &Transaction) {
        for (i, out) in tx.outputs.iter().enumerate() {
            let op = OutPoint {
                txid: tx.txid.clone(),
                vout: i as u32,
            };
            self.insert(op, out.clone());
        }
    }
}
