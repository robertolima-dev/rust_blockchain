use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::utxo::OutPoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    /// References a previous unspent output (UTXO)
    pub outpoint: OutPoint,
    // Placeholder for signatures (to be implemented later)
    // pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub address: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// A stable identifier computed from content.
    pub txid: String,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

impl Transaction {
    /// Build a transaction and compute its txid deterministically from its content.
    pub fn new(mut inputs: Vec<TxInput>, mut outputs: Vec<TxOutput>) -> Self {
        let payload = serde_json::json!({
            "inputs": inputs,
            "outputs": outputs,
        });
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_vec(&payload).expect("json serialize"));
        let txid = hex::encode(hasher.finalize());

        Self {
            txid,
            inputs: inputs.drain(..).collect(),
            outputs: outputs.drain(..).collect(),
        }
    }

    pub fn total_output_amount(&self) -> u128 {
        self.outputs.iter().map(|o| o.amount as u128).sum()
    }
}
