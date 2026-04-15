//! Transaction Pool Module

use alloy_rpc_types::Transaction;
use std::collections::VecDeque;

/// Transaction pool
pub struct TxPool {
    queue: VecDeque<Transaction>,
}

impl TxPool {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
    
    pub fn add(&mut self, tx: Transaction) {
        self.queue.push_back(tx);
    }
    
    pub fn pop(&mut self) -> Option<Transaction> {
        self.queue.pop_front()
    }
    
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Default for TxPool {
    fn default() -> Self {
        Self::new()
    }
}
