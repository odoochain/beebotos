//! Knowledge Module
//!
//! Knowledge representation and reasoning systems.

pub mod concept_net;
pub mod fusion;
pub mod graph;
pub mod inference;
pub mod ontology;

pub use graph::{Edge, KnowledgeGraph, Node};
pub use ontology::{Concept, Ontology};

/// Knowledge engine
pub struct KnowledgeEngine {
    graph: KnowledgeGraph,
    ontology: Ontology,
}

impl KnowledgeEngine {
    pub fn new() -> Self {
        Self {
            graph: KnowledgeGraph::new(),
            ontology: Ontology::new(),
        }
    }

    pub fn graph(&self) -> &KnowledgeGraph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut KnowledgeGraph {
        &mut self.graph
    }

    pub fn ontology(&self) -> &Ontology {
        &self.ontology
    }
}

impl Default for KnowledgeEngine {
    fn default() -> Self {
        Self::new()
    }
}
