use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::utxo::OutPoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    /// References a previous unspent output (UTXO)
    pub outpoint: OutPoint,
    // Placeholder for signatures (to be implemented later)
    // pub signature: String,
    pub pubkey: String,
    /// Hex-encoded DER ECDSA signature
    pub signature: String,
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

// impl Transaction {
//     /// Build a transaction and compute its txid deterministically from its content.
//     pub fn new(mut inputs: Vec<TxInput>, mut outputs: Vec<TxOutput>) -> Self {
//         let payload = serde_json::json!({
//             "inputs": inputs,
//             "outputs": outputs,
//         });
//         let mut hasher = Sha256::new();
//         hasher.update(serde_json::to_vec(&payload).expect("json serialize"));
//         let txid = hex::encode(hasher.finalize());

//         Self {
//             txid,
//             inputs: inputs.drain(..).collect(),
//             outputs: outputs.drain(..).collect(),
//         }
//     }

//     pub fn total_output_amount(&self) -> u128 {
//         self.outputs.iter().map(|o| o.amount as u128).sum()
//     }
// }

impl Transaction {
    /// Build a transaction and compute its txid deterministically from its content.
    /// TXID includes signatures; SIGHASH (used for signing) excludes signatures/pubkeys.
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

    /// Canonical signing payload (JSON) that excludes signatures and pubkeys.
    /// This is what should be hashed and signed by each input's owner.
    pub fn signing_payload(&self) -> Vec<u8> {
        // Only the outpoints (txid, vout) and outputs are included
        let lite_inputs: Vec<_> = self
            .inputs
            .iter()
            .map(|i| serde_json::json!({ "txid": i.outpoint.txid, "vout": i.outpoint.vout }))
            .collect();
        let payload = serde_json::json!({
            "inputs": lite_inputs,
            "outputs": self.outputs,
        });
        serde_json::to_vec(&payload).expect("serialize signing payload")
    }

    /// SHA-256 of the signing payload.
    pub fn sighash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.signing_payload());
        let digest = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest[..]);
        out
    }
}
