//! Agent Evolution Example
//!
//! Demonstrates a complete NEAT evolution workflow for agent neural networks.

use beebotos_brain::{
    Emotion, EmotionalIntelligence, FitnessResult, Genome, InnovationTracker, NeatConfig,
    NeuralNetwork, OceanProfile, Pad, Population,
};

fn main() {
    println!("🧬 Agent Evolution Example");
    println!("===========================\n");

    // Configure NEAT parameters
    let config = NeatConfig::standard();
    println!("📊 NEAT Configuration:");
    println!("   - Population size: {}", config.population_size);
    println!("   - Mutation rate: {:.2}", config.mutation_rate);
    println!(
        "   - Add node probability: {:.2}",
        config.add_node_probability
    );
    println!(
        "   - Add connection probability: {:.2}\n",
        config.add_connection_probability
    );

    // Create initial population
    let input_size = 4; // [energy, threat_distance, resource_distance, social_signal]
    let output_size = 3; // [move_toward, flee, interact]

    let mut population = Population::new(config.population_size, input_size, output_size, &config);

    // Create agent with personality and emotional intelligence
    let personality = OceanProfile::analytical();
    let mut emotional_intelligence = EmotionalIntelligence::new();

    println!("🎯 Evolution Target:");
    println!(
        "   - Inputs: {:?}",
        vec!["energy", "threat", "resource", "social"]
    );
    println!("   - Outputs: {:?}", vec!["approach", "flee", "interact"]);
    println!("   - Personality: Analytical\n");

    // Evolution loop
    let generations = 50;
    let mut best_fitness_history = Vec::new();

    for generation in 0..generations {
        // Evaluate fitness for each genome
        let fitness_results: Vec<FitnessResult> = population
            .genomes
            .iter()
            .map(|genome| {
                let fitness = evaluate_agent(genome, input_size, output_size);

                FitnessResult {
                    agent_id: beebotos_core::AgentId::new(),
                    fitness,
                    generation,
                    metrics: std::collections::HashMap::new(),
                }
            })
            .collect();

        // Record best fitness
        let best_fitness = fitness_results
            .iter()
            .map(|r| r.fitness)
            .fold(f32::NEG_INFINITY, f32::max);
        best_fitness_history.push(best_fitness);

        // Evolve to next generation
        population.evolve(&fitness_results, &config);

        // Update emotional state based on performance
        let avg_fitness =
            fitness_results.iter().map(|r| r.fitness).sum::<f32>() / fitness_results.len() as f32;

        let emotion_impact = if avg_fitness > 0.5 {
            Pad::new(0.3, 0.2, 0.1) // Positive emotion
        } else {
            Pad::new(-0.2, 0.3, -0.1) // Negative emotion
        };

        // Print progress every 10 generations
        if generation % 10 == 0 || generation == generations - 1 {
            let stats = population.stats();
            println!(
                "📈 Generation {:3}: Best={:.4}, Avg={:.4}, Species={}, Complexity={}",
                stats.generation,
                stats.max_fitness,
                stats.avg_fitness,
                stats.species_count,
                population
                    .best_genome
                    .as_ref()
                    .map(|g| g.node_count())
                    .unwrap_or(0)
            );
        }
    }

    // Final results
    println!("\n✅ Evolution Complete!");
    println!("=======================");

    if let Some(best_genome) = &population.best_genome {
        println!("\n🏆 Best Genome:");
        println!("   - ID: {}", best_genome.id);
        println!("   - Layers: {}", best_genome.layers.len());
        println!("   - Connections: {}", best_genome.connections.len());
        println!(
            "   - Enabled connections: {}",
            best_genome.connections.iter().filter(|c| c.enabled).count()
        );
        println!("   - Final fitness: {:.4}", population.best_fitness);

        // Test the best network
        let network = NeuralNetwork::from_genome(best_genome);
        println!("\n🧠 Testing Best Agent:");

        let test_scenarios = vec![
            ("Low energy, no threat", vec![0.2, 0.0, 0.8, 0.0]),
            ("High energy, threat near", vec![0.9, 0.8, 0.0, 0.0]),
            ("Social opportunity", vec![0.6, 0.0, 0.0, 0.9]),
        ];

        for (scenario, inputs) in test_scenarios {
            let outputs = network.predict(&inputs);
            let decision = match outputs
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            {
                Some((0, _)) => "APPROACH",
                Some((1, _)) => "FLEE",
                Some((2, _)) => "INTERACT",
                _ => "UNKNOWN",
            };
            println!(
                "   {} -> Decision: {} (outputs: {:?})",
                scenario,
                decision,
                outputs
                    .iter()
                    .map(|o| format!("{:.2}", o))
                    .collect::<Vec<_>>()
            );
        }
    }

    // Show fitness progression
    println!("\n📊 Fitness Progression:");
    print!("   ");
    for (i, fitness) in best_fitness_history.iter().step_by(10).enumerate() {
        print!("G{}:{:.2} ", i * 10, fitness);
    }
    println!();

    println!("\n✨ Example complete!");
}

/// Evaluate agent fitness
fn evaluate_agent(genome: &Genome, input_size: usize, output_size: usize) -> f32 {
    let network = NeuralNetwork::from_genome(genome);

    // Test scenarios with expected behaviors
    let test_cases = vec![
        // (input, expected_output, importance)
        (vec![0.2, 0.0, 0.8, 0.0], vec![1.0, 0.0, 0.0], 1.0), // Low energy -> approach resource
        (vec![0.9, 0.8, 0.0, 0.0], vec![0.0, 1.0, 0.0], 1.0), // Threat near -> flee
        (vec![0.6, 0.0, 0.0, 0.9], vec![0.0, 0.0, 1.0], 0.8), // Social signal -> interact
        (vec![0.5, 0.5, 0.5, 0.5], vec![0.3, 0.3, 0.4], 0.5), // Balanced -> moderate action
    ];

    let mut total_fitness = 0.0;
    let mut total_weight = 0.0;

    for (input, expected, weight) in test_cases {
        let output = network.predict(&input);

        // Calculate error
        let error: f32 = output
            .iter()
            .zip(expected.iter())
            .map(|(o, e)| (o - e).powi(2))
            .sum();

        // Convert to fitness (inverse of error)
        let case_fitness = 1.0 / (1.0 + error);
        total_fitness += case_fitness * weight;
        total_weight += weight;
    }

    // Normalize by total weight
    let base_fitness = total_fitness / total_weight;

    // Bonus for network efficiency (prefer simpler networks)
    let enabled_connections = genome.connections.iter().filter(|c| c.enabled).count() as f32;
    let efficiency_bonus = 1.0 / (1.0 + enabled_connections / 100.0);

    base_fitness * 0.9 + efficiency_bonus * 0.1
}
