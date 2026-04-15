//! Mutation operations for NEAT

use super::genome::{Genome, Connection, Node};
use rand::Rng;

pub struct Mutation;

impl Mutation {
    /// Mutate a genome
    pub fn mutate_genome(genome: &mut Genome) {
        let mut rng = rand::thread_rng();
        
        // Weight mutation
        if rng.gen_bool(0.8) {
            Self::mutate_weights(genome);
        }
        
        // Add connection
        if rng.gen_bool(0.05) {
            Self::add_connection(genome);
        }
        
        // Add node
        if rng.gen_bool(0.03) {
            Self::add_node(genome);
        }
        
        // Toggle connection
        if rng.gen_bool(0.01) {
            Self::toggle_connection(genome);
        }
    }
    
    /// Mutate connection weights
    fn mutate_weights(genome: &mut Genome) {
        let mut rng = rand::thread_rng();
        
        for conn in &mut genome.connections {
            if rng.gen_bool(0.9) {
                // Perturb weight
                let perturbation = rng.gen_range(-0.1..0.1);
                conn.weight += perturbation;
            } else {
                // Assign new random weight
                conn.weight = rng.gen_range(-4.0..4.0);
            }
            
            // Clamp weight
            conn.weight = conn.weight.clamp(-30.0, 30.0);
        }
    }
    
    /// Add a new connection
    fn add_connection(genome: &mut Genome) {
        let mut rng = rand::thread_rng();
        
        if genome.nodes.len() < 2 {
            return;
        }
        
        // Try to find valid connection
        for _ in 0..20 {
            let from_idx = rng.gen_range(0..genome.nodes.len());
            let to_idx = rng.gen_range(0..genome.nodes.len());
            
            if from_idx == to_idx {
                continue;
            }
            
            let from_node = &genome.nodes[from_idx];
            let to_node = &genome.nodes[to_idx];
            
            // Don't connect to input nodes, don't connect from output nodes
            if to_node.is_input || from_node.is_output {
                continue;
            }
            
            // Check if connection already exists
            let exists = genome.connections.iter()
                .any(|c| c.from_node == from_node.id && c.to_node == to_node.id);
            
            if !exists {
                let innovation = genome.connections.len() as u64 + 1;
                let weight = rng.gen_range(-1.0..1.0);
                genome.add_connection(from_idx, to_idx, innovation);
                break;
            }
        }
    }
    
    /// Add a new node (split a connection)
    fn add_node(genome: &mut Genome) {
        let mut rng = rand::thread_rng();
        
        if genome.connections.is_empty() {
            return;
        }
        
        // Select random enabled connection
        let enabled_connections: Vec<usize> = genome.connections.iter()
            .enumerate()
            .filter(|(_, c)| c.enabled)
            .map(|(i, _)| i)
            .collect();
        
        if enabled_connections.is_empty() {
            return;
        }
        
        let conn_idx = enabled_connections[rng.gen_range(0..enabled_connections.len())];
        let conn = &genome.connections[conn_idx];
        
        let from_node = conn.from_node;
        let to_node = conn.to_node;
        let old_weight = conn.weight;
        
        // Disable old connection
        genome.connections[conn_idx].enabled = false;
        
        // Add new node
        let new_node_id = genome.nodes.len();
        genome.nodes.push(Node {
            id: new_node_id,
            is_input: false,
            is_output: false,
            activation: crate::neat::genome::ActivationFunction::Sigmoid,
        });
        
        // Add two new connections
        let innovation1 = genome.connections.len() as u64 + 1;
        let innovation2 = innovation1 + 1;
        
        genome.add_connection_by_id(from_node, new_node_id, innovation1);
        genome.add_connection_by_id(new_node_id, to_node, innovation2);
        
        // Set weights
        if let Some(last) = genome.connections.last_mut() {
            last.weight = old_weight;
        }
    }
    
    /// Toggle connection on/off
    fn toggle_connection(genome: &mut Genome) {
        let mut rng = rand::thread_rng();
        
        if let Some(conn) = genome.connections.iter_mut().choose(&mut rng) {
            conn.enabled = !conn.enabled;
        }
    }
}
