//! Chain Types

use crate::compat::{Address, U256, B256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: B256,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub data: Vec<u8>,
    pub gas_price: U256,
    pub gas_limit: U256,
    pub nonce: u64,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub number: u64,
    pub hash: B256,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
}
