//! Indexer Queries

use crate::compat::B256;

/// Query filter for indexed data
#[derive(Debug, Clone)]
pub struct QueryFilter {
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
    pub topics: Vec<Option<Vec<B256>>>,
}

/// Query result
#[derive(Debug, Clone)]
pub struct QueryResult<T> {
    pub data: Vec<T>,
    pub total_count: u64,
}

/// Query builder
pub struct QueryBuilder {
    filter: QueryFilter,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            filter: QueryFilter {
                from_block: None,
                to_block: None,
                topics: Vec::new(),
            },
        }
    }
    
    pub fn from_block(mut self, block: u64) -> Self {
        self.filter.from_block = Some(block);
        self
    }
    
    pub fn to_block(mut self, block: u64) -> Self {
        self.filter.to_block = Some(block);
        self
    }
    
    pub fn build(self) -> QueryFilter {
        self.filter
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
