# NEAT Evolution Specification

## Overview

NeuroEvolution of Augmenting Topologies (NEAT) is a genetic algorithm for evolving artificial neural networks.

## Core Concepts

### Genomes

A genome represents a neural network topology:

```rust
struct Genome {
    nodes: Vec<NodeGene>,
    connections: Vec<ConnectionGene>,
    fitness: f64,
}

struct NodeGene {
    id: usize,
    node_type: NodeType,  // Input, Hidden, Output
    activation: ActivationFunction,
}

struct ConnectionGene {
    in_node: usize,
    out_node: usize,
    weight: f64,
    enabled: bool,
    innovation: usize,
}
```

### Innovation Tracking

Each structural mutation is assigned a global innovation number:

```rust
struct InnovationHistory {
    innovations: Vec<Innovation>,
    current_innovation: usize,
}

struct Innovation {
    innovation_number: usize,
    in_node: usize,
    out_node: usize,
}
```

## Evolution Process

### 1. Speciation

Organize genomes into species based on compatibility:

```
compatibility = (c1 * E/N) + (c2 * D/N) + (c3 * W)

Where:
- E = number of excess genes
- D = number of disjoint genes
- W = average weight difference of matching genes
- N = number of genes in larger genome
- c1, c2, c3 = coefficients
```

### 2. Selection

```rust
fn select_genome(species: &Species) -> &Genome {
    // Tournament selection
    let tournament_size = 3;
    let mut best = random_genome(species);
    
    for _ in 1..tournament_size {
        let contender = random_genome(species);
        if contender.fitness > best.fitness {
            best = contender;
        }
    }
    
    best
}
```

### 3. Crossover

```rust
fn crossover(parent1: &Genome, parent2: &Genome) -> Genome {
    let mut child = Genome::new();
    let (better, worse) = if parent1.fitness > parent2.fitness {
        (parent1, parent2)
    } else {
        (parent2, parent1)
    };
    
    // Inherit matching genes randomly
    for (gene1, gene2) in matching_genes(better, worse) {
        child.add_connection(
            if random() < 0.5 { gene1 } else { gene2 }
        );
    }
    
    // Inherit disjoint and excess genes from better parent
    for gene in disjoint_and_excess_genes(better, worse) {
        child.add_connection(gene.clone());
    }
    
    child
}
```

### 4. Mutation

#### Weight Mutation
```rust
fn mutate_weights(genome: &mut Genome, config: &Config) {
    for conn in &mut genome.connections {
        if random() < config.weight_mutation_rate {
            if random() < config.perturbation_rate {
                // Perturb weight
                conn.weight += random_gaussian() * config.step_size;
            } else {
                // Assign new random weight
                conn.weight = random_uniform(-1.0, 1.0);
            }
        }
    }
}
```

#### Add Connection
```rust
fn add_connection_mutation(genome: &mut Genome, innovation: &mut InnovationHistory) {
    let (in_node, out_node) = random_unconnected_pair(genome);
    
    let innovation_num = innovation.get_number(in_node, out_node);
    
    genome.add_connection(ConnectionGene {
        in_node,
        out_node,
        weight: random_uniform(-1.0, 1.0),
        enabled: true,
        innovation: innovation_num,
    });
}
```

#### Add Node
```rust
fn add_node_mutation(genome: &mut Genome, innovation: &mut InnovationHistory) {
    // Split existing connection
    let conn = random_enabled_connection(genome);
    conn.enabled = false;
    
    let new_node_id = genome.add_node(NodeType::Hidden);
    
    // Add two new connections
    let inno1 = innovation.get_number(conn.in_node, new_node_id);
    genome.add_connection(ConnectionGene {
        in_node: conn.in_node,
        out_node: new_node_id,
        weight: 1.0,
        enabled: true,
        innovation: inno1,
    });
    
    let inno2 = innovation.get_number(new_node_id, conn.out_node);
    genome.add_connection(ConnectionGene {
        in_node: new_node_id,
        out_node: conn.out_node,
        weight: conn.weight,
        enabled: true,
        innovation: inno2,
    });
}
```

## Configuration

```rust
struct Config {
    // Population
    population_size: usize,
    
    // Speciation
    compatibility_threshold: f64,
    excess_coefficient: f64,
    disjoint_coefficient: f64,
    weight_coefficient: f64,
    
    // Mutation rates
    weight_mutation_rate: f64,
    add_connection_rate: f64,
    add_node_rate: f64,
    
    // Mutation parameters
    perturbation_rate: f64,
    step_size: f64,
    
    // Stagnation
    stagnation_threshold: usize,
    
    // Elitism
    elitism: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            population_size: 150,
            compatibility_threshold: 3.0,
            excess_coefficient: 1.0,
            disjoint_coefficient: 1.0,
            weight_coefficient: 0.4,
            weight_mutation_rate: 0.8,
            add_connection_rate: 0.05,
            add_node_rate: 0.03,
            perturbation_rate: 0.9,
            step_size: 0.1,
            stagnation_threshold: 15,
            elitism: true,
        }
    }
}
```

## Fitness Evaluation

```rust
trait FitnessFunction {
    fn evaluate(&self, genome: &Genome) -> f64;
}

struct XORFitness;

impl FitnessFunction for XORFitness {
    fn evaluate(&self, genome: &Genome) -> f64 {
        let inputs = vec![
            (0.0, 0.0),
            (0.0, 1.0),
            (1.0, 0.0),
            (1.0, 1.0),
        ];
        let expected = vec![0.0, 1.0, 1.0, 0.0];
        
        let mut error = 0.0;
        for ((x, y), exp) in inputs.iter().zip(expected.iter()) {
            let output = genome.activate(&vec![*x, *y])[0];
            error += (output - exp).powi(2);
        }
        
        4.0 - error  // Higher is better
    }
}
```

## Network Activation

```rust
impl Genome {
    fn activate(&self, inputs: &[f64]) -> Vec<f64> {
        let mut node_values: HashMap<usize, f64> = HashMap::new();
        
        // Set input values
        for (i, &value) in inputs.iter().enumerate() {
            node_values.insert(i, value);
        }
        
        // Calculate hidden and output nodes
        for node in &self.nodes {
            if node.node_type == NodeType::Hidden || node.node_type == NodeType::Output {
                let sum: f64 = self.connections
                    .iter()
                    .filter(|c| c.out_node == node.id && c.enabled)
                    .map(|c| {
                        node_values.get(&c.in_node).unwrap_or(&0.0) * c.weight
                    })
                    .sum();
                
                node_values.insert(node.id, node.activation.apply(sum));
            }
        }
        
        // Collect outputs
        self.nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Output)
            .map(|n| node_values.get(&n.id).copied().unwrap_or(0.0))
            .collect()
    }
}
```

## References

- Stanley, K. O., & Miikkulainen, R. (2002). Evolving Neural Networks through Augmenting Topologies. Evolutionary Computation, 10(2), 99-127.
