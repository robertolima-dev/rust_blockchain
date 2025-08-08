use super::Block;

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

    /// Mine and append a new block with the provided `data`.
    pub fn mine_block(&mut self, data: String) -> &Block {
        let index = self.chain.len() as u64;
        let prev_hash = self.last_block().hash.clone();

        let mut block = Block::new(index, prev_hash, data);
        block.mine(self.difficulty);

        self.chain.push(block);
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
        // NOTE: Changing difficulty affects future blocks only.
        self.difficulty = difficulty;
    }
}
