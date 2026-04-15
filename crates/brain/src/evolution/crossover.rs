//! Crossover operations for NEAT

use super::genome::Genome;
use rand::Rng;

pub struct Crossover;

impl Crossover {
    /// Perform crossover between two parent genomes
    pub fn crossover(parent1: &Genome, parent2: &Genome, _next_innovation: impl FnMut() -> u64) -> Genome {
        let mut rng = rand::thread_rng();
        
        // Determine fitter parent
        let (fitter, weaker) = if parent1.fitness >= parent2.fitness {
            (parent1, parent2)
        } else {
            (parent2, parent1)
        };
        
        let mut child = Genome::new(fitter.inputs, fitter.outputs);
        
        // Inherit matching genes randomly
        for conn1 in &fitter.connections {
            if let Some(conn2) = weaker.connections.iter().find(|c| c.innovation == conn1.innovation) {
                // Matching gene - random choice
                let chosen = if rng.gen_bool(0.5) { conn1 } else { conn2 };
                child.connections.push(chosen.clone());
            } else {
                // Disjoint/excess gene from fitter parent
                child.connections.push(conn1.clone());
            }
        }
        
        // Inherit nodes
        child.nodes = fitter.nodes.clone();
        
        child
    }
    
    /// Uniform crossover (alternative)
    pub fn uniform_crossover(parent1: &Genome, parent2: &Genome) -> Genome {
        let mut rng = rand::thread_rng();
        let mut child = Genome::new(parent1.inputs, parent1.outputs);
        
        // Combine all unique innovations
        let mut innovations: Vec<u64> = parent1.connections.iter()
            .map(|c| c.innovation)
            .collect();
        
        for conn in &parent2.connections {
            if !innovations.contains(&conn.innovation) {
                innovations.push(conn.innovation);
            }
        }
        
        // Randomly select from either parent
        for innovation in innovations {
            let conn1 = parent1.connections.iter().find(|c| c.innovation == innovation);
            let conn2 = parent2.connections.iter().find(|c| c.innovation == innovation);
            
            match (conn1, conn2) {
                (Some(c1), Some(_)) => {
                    if rng.gen_bool(0.5) {
                        child.connections.push(c1.clone());
                    } else if let Some(c2) = conn2 {
                        child.connections.push(c2.clone());
                    }
                }
                (Some(c1), None) => child.connections.push(c1.clone()),
                (None, Some(c2)) => child.connections.push(c2.clone()),
                _ => {}
            }
        }
        
        child.nodes = parent1.nodes.clone();
        child
    }
}
