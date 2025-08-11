pub mod model;
pub mod utxo;

pub use model::{Transaction, TxInput, TxOutput};
pub use utxo::{OutPoint, UtxoSet};
