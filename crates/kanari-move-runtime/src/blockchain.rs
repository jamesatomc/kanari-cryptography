use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub timestamp: u64,
    pub prev_hash: Vec<u8>,
    pub state_root: Vec<u8>,
    pub tx_count: usize,
}

impl BlockHeader {
    pub fn new(height: u64, prev_hash: Vec<u8>, state_root: Vec<u8>, tx_count: usize) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            height,
            timestamp,
            prev_hash,
            state_root,
            tx_count,
        }
    }

    pub fn hash(&self) -> Vec<u8> {
        let serialized = serde_json::to_vec(self).unwrap();
        Sha256::digest(&serialized).to_vec()
    }
}

/// Transaction types in Kanari blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transaction {
    /// Publish a Move module
    PublishModule {
        sender: String,
        module_bytes: Vec<u8>,
        module_name: String,
        gas_limit: u64,
        gas_price: u64,
    },
    /// Execute a Move function
    ExecuteFunction {
        sender: String,
        module: String,
        function: String,
        type_args: Vec<String>,
        args: Vec<Vec<u8>>,
        gas_limit: u64,
        gas_price: u64,
    },
    /// Transfer coins
    Transfer {
        from: String,
        to: String,
        amount: u64,
        gas_limit: u64,
        gas_price: u64,
    },
}

impl Transaction {
    pub fn hash(&self) -> Vec<u8> {
        let serialized = serde_json::to_vec(self).unwrap();
        Sha256::digest(&serialized).to_vec()
    }

    pub fn sender(&self) -> &str {
        match self {
            Transaction::PublishModule { sender, .. } => sender,
            Transaction::ExecuteFunction { sender, .. } => sender,
            Transaction::Transfer { from, .. } => from,
        }
    }

    pub fn gas_limit(&self) -> u64 {
        match self {
            Transaction::PublishModule { gas_limit, .. } => *gas_limit,
            Transaction::ExecuteFunction { gas_limit, .. } => *gas_limit,
            Transaction::Transfer { gas_limit, .. } => *gas_limit,
        }
    }

    pub fn gas_price(&self) -> u64 {
        match self {
            Transaction::PublishModule { gas_price, .. } => *gas_price,
            Transaction::ExecuteFunction { gas_price, .. } => *gas_price,
            Transaction::Transfer { gas_price, .. } => *gas_price,
        }
    }

    /// Create a transfer transaction with default gas settings
    pub fn new_transfer(from: String, to: String, amount: u64) -> Self {
        Self::Transfer {
            from,
            to,
            amount,
            gas_limit: 100_000,  // Default gas limit
            gas_price: 1000,     // Default gas price (1000 Mist)
        }
    }
}

/// Block containing transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(height: u64, prev_hash: Vec<u8>, transactions: Vec<Transaction>) -> Self {
        let state_root = vec![0u8; 32]; // Placeholder, compute from state
        let tx_count = transactions.len();
        let header = BlockHeader::new(height, prev_hash, state_root, tx_count);

        Self {
            header,
            transactions,
        }
    }

    pub fn genesis() -> Self {
        Self::new(0, vec![0u8; 32], vec![])
    }

    pub fn hash(&self) -> Vec<u8> {
        self.header.hash()
    }

    pub fn verify(&self, prev_block: &Block) -> Result<()> {
        // Verify height
        if self.header.height != prev_block.header.height + 1 {
            anyhow::bail!("Invalid block height");
        }

        // Verify prev_hash
        if self.header.prev_hash != prev_block.hash() {
            anyhow::bail!("Invalid previous hash");
        }

        // Verify timestamp (allow some leeway for genesis)
        if self.header.height > 1 && self.header.timestamp < prev_block.header.timestamp {
            anyhow::bail!("Invalid timestamp");
        }

        Ok(())
    }
}

/// Blockchain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::genesis();
        Self {
            blocks: vec![genesis],
        }
    }

    pub fn latest_block(&self) -> &Block {
        self.blocks.last().unwrap()
    }

    pub fn height(&self) -> u64 {
        self.latest_block().header.height
    }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        let prev_block = self.latest_block();
        block.verify(prev_block)?;
        self.blocks.push(block);
        Ok(())
    }

    pub fn get_block(&self, height: u64) -> Option<&Block> {
        self.blocks.iter().find(|b| b.header.height == height)
    }

    pub fn get_transaction_count(&self) -> usize {
        self.blocks.iter().map(|b| b.transactions.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.header.height, 0);
        assert_eq!(genesis.transactions.len(), 0);
    }

    #[test]
    fn test_blockchain_creation() {
        let chain = Blockchain::new();
        assert_eq!(chain.height(), 0);
        assert_eq!(chain.blocks.len(), 1);
    }

    #[test]
    fn test_add_block() {
        let mut chain = Blockchain::new();
        let prev_hash = chain.latest_block().hash();
        
        let block = Block::new(1, prev_hash, vec![]);
        chain.add_block(block).unwrap();
        
        assert_eq!(chain.height(), 1);
        assert_eq!(chain.blocks.len(), 2);
    }

    #[test]
    fn test_block_verification() {
        let chain = Blockchain::new();
        let prev_block = chain.latest_block();
        
        let valid_block = Block::new(1, prev_block.hash(), vec![]);
        assert!(valid_block.verify(prev_block).is_ok());
        
        let invalid_block = Block::new(2, prev_block.hash(), vec![]);
        assert!(invalid_block.verify(prev_block).is_err());
    }

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction::new_transfer(
            "0x1".to_string(),
            "0x2".to_string(),
            1000,
        );
        
        let hash1 = tx.hash();
        let hash2 = tx.hash();
        assert_eq!(hash1, hash2);
    }
}
