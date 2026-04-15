# BeeBotOS Brain Module

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

The BeeBotOS Brain module provides a comprehensive cognitive architecture for AI agents, implementing advanced neural evolution, emotional modeling, personality systems, and multi-modal memory.

## Features

### 🧠 Core Capabilities

- **NEAT Neural Evolution** - NeuroEvolution of Augmenting Topologies for evolving neural network structures
- **PAD Emotional Model** - Pleasure-Arousal-Dominance emotional state representation
- **OCEAN Personality** - Big Five personality trait modeling and adaptation
- **Multi-Modal Memory** - Short-term, episodic, semantic, and procedural memory systems
- **Metacognition** - Self-reflection, performance monitoring, and strategy adjustment

### 🎯 Advanced Features

- **Attention Mechanisms** - Focus, saliency computation, and selective attention
- **Learning Systems** - Q-Learning, policy gradients, and skill acquisition
- **Reasoning Engines** - Deductive, inductive, and abductive reasoning
- **Knowledge Graphs** - Structured knowledge representation and inference
- **Social Cognition** - Relationship modeling, trust, and reputation systems
- **Creativity Engine** - Idea generation, brainstorming, and solution synthesis

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
beebotos-brain = "0.1.0"
```

### Basic Usage

```rust
use beebotos_brain::prelude::*;

fn main() -> BrainResult<()> {
    // Create a new brain instance
    let mut api = SocialBrainApi::new();
    
    // Process a stimulus
    let response = api.process_stimulus("Hello, world!")?;
    println!("Response: {}", response.response);
    
    // Set a goal
    let goal_id = api.set_goal("Complete task", 0.8)?;
    println!("Goal created: {}", goal_id);
    
    // Check current emotional state
    let emotion = api.current_emotion();
    println!("Current mood: {:?}", emotion);
    
    Ok(())
}
```

### Using Custom Configuration

```rust
use beebotos_brain::{BrainConfig, ApiConfig, SocialBrainApi};

// Create lightweight configuration
let config = BrainConfig::lightweight();
let api = SocialBrainApi::with_config(ApiConfig::default());

// Or high-performance configuration
let config = BrainConfig::high_performance();
```

## Architecture

### Module Structure

```
beebotos-brain/
├── api/           # Public API interface
├── attention/     # Attention and focus mechanisms
├── cognition/     # Cognitive state and working memory
├── creativity/    # Idea generation and brainstorming
├── emotion/       # Emotional state and dynamics
├── error/         # Error types and handling
├── evolution/     # Evolutionary algorithms
├── knowledge/     # Knowledge representation
├── language/      # NLP and sentiment analysis
├── learning/      # Reinforcement learning
├── memory/        # Multi-modal memory systems
├── metacognition/ # Self-reflection and monitoring
├── neat/          # NEAT neural evolution
├── pad/           # PAD emotional model
├── personality/   # OCEAN personality model
├── reasoning/     # Logical reasoning engines
├── social/        # Social cognition
└── utils/         # Utility functions
```

### Key Components

#### NEAT (NeuroEvolution of Augmenting Topologies)

Evolves neural network topology and weights simultaneously:

```rust
use beebotos_brain::{Genome, NeatConfig, NeuralNetwork};

// Create initial genome
let genome = Genome::new_minimal(1, 3, 2);  // id, inputs, outputs

// Build network from genome
let network = NeuralNetwork::from_genome(&genome);

// Activate network
let outputs = network.predict(&[0.5, 0.3, 0.2]);
```

#### PAD Emotional Model

3-dimensional emotional state representation:

```rust
use beebotos_brain::Pad;

// Create emotional state
let happy = Pad::new(0.8, 0.6, 0.4);  // pleasure, arousal, dominance
let sad = Pad::new(-0.8, -0.4, -0.4);

// Blend emotions
let mixed = happy.lerp(&sad, 0.3);
```

#### Memory Systems

```rust
use beebotos_brain::{
    ShortTermMemory, EpisodicMemory, SemanticMemory,
    MemoryQuery, MemoryType
};

// Short-term memory (7±2 items)
let mut stm = ShortTermMemory::new();
stm.push("Important information");

// Episodic memory (events with context)
let mut episodic = EpisodicMemory::new();
episodic.encode("Met Alice at coffee shop", timestamp, Some(location));

// Semantic memory (concepts and facts)
let mut semantic = SemanticMemory::new();
semantic.learn_concept("Rust", "Systems programming language", "Programming");

// Query across memory types
let query = MemoryQuery::new("coffee shop")
    .with_types(vec![MemoryType::Episodic, MemoryType::Semantic])
    .with_limit(5);
```

#### Personality (OCEAN Model)

```rust
use beebotos_brain::OceanProfile;

// Create personality profile
let profile = OceanProfile::new(
    0.7,  // Openness
    0.5,  // Conscientiousness
    0.6,  // Extraversion
    0.8,  // Agreeableness
    0.3,  // Neuroticism (low = emotionally stable)
);

// Or use presets
let creative = OceanProfile::creative();
let analytical = OceanProfile::analytical();
let social = OceanProfile::social();
```

## Configuration

### Configuration Presets

```rust
use beebotos_brain::BrainConfig;

// Standard configuration (balanced)
let standard = BrainConfig::standard();

// Lightweight (fewer features, lower resource usage)
let lightweight = BrainConfig::lightweight();

// High-performance (all features enabled)
let high_perf = BrainConfig::high_performance();
```

### Feature Toggles

```rust
use beebotos_brain::FeatureToggles;

let features = FeatureToggles {
    learning: true,        // Enable learning module
    social: true,          // Enable social cognition
    metacognition: true,   // Enable self-reflection
    creativity: true,      // Enable creativity module
    detailed_logging: false,
};
```

## Examples

### Emotional Intelligence

```rust
use beebotos_brain::{EmotionalIntelligence, EmotionalEvent, Pad};

let mut ei = EmotionalIntelligence::new();

ei.update(&EmotionalEvent {
    description: "Received good news".to_string(),
    pleasure_impact: 0.6,
    arousal_impact: 0.4,
    dominance_impact: 0.2,
});

let current = ei.current();
println!("Pleasure: {}, Arousal: {}", current.pleasure, current.arousal);
```

### Q-Learning

```rust
use beebotos_brain::QLearning;

let mut agent = QLearning::new();

// Choose action
let action = agent.choose_action("state1", &["action1", "action2", "action3"]);

// Update after receiving reward
agent.update("state1", &action, 1.0, "state2", &["action1", "action2"]);
```

### Knowledge Base

```rust
use beebotos_brain::{
    KnowledgeBase, Fact, Rule, Atom, Term
};

let mut kb = KnowledgeBase::new();

// Add facts
kb.add_fact(Fact::new("human").with_arg(Term::const_("socrates")));

// Add rules
let rule = Rule::new(Atom::new("mortal").arg(Term::var("X")))
    .if_(Atom::new("human").arg(Term::var("X")));
kb.add_rule(rule);

// Forward chaining inference
let new_facts = kb.forward_chain(10);
```

## Performance

### Benchmarks

Run benchmarks with:

```bash
cargo bench
```

Key performance metrics:

| Operation | Complexity | Typical Time |
|-----------|-----------|--------------|
| Memory Index Search | O(1) avg | < 1 μs |
| NEAT Activation | O(n) | ~10 μs |
| PAD Update | O(1) | < 100 ns |
| STM Push | O(1) | < 1 μs |

### Memory Usage

Approximate memory usage per component:

- ShortTermMemory: ~1KB base + ~100B per item
- EpisodicMemory: ~10KB base + ~500B per episode
- Genome (NEAT): ~1-10KB depending on complexity
- NeuralNetwork: ~10-100KB depending on topology

## API Reference

### Main Types

| Type | Description |
|------|-------------|
| `SocialBrainApi` | Main API interface |
| `BrainResult<T>` | Result type with BrainError |
| `BrainError` | Error enum for all operations |
| `Pad` | PAD emotional state |
| `OceanProfile` | OCEAN personality profile |
| `Genome` | NEAT genetic encoding |
| `NeuralNetwork` | Phenotypic neural network |

### Error Handling

```rust
use beebotos_brain::{BrainResult, BrainError};

fn example() -> BrainResult<i32> {
    // Automatic error conversion with ?
    let file = std::fs::read_to_string("config.json")?;
    let data: i32 = file.parse().map_err(|_| "parse failed")?;
    
    // Create errors
    return Err(BrainError::InvalidParameter("bad input".to_string()));
}
```

## Development

### Building

```bash
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin

# Run benchmarks
cargo bench
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check documentation
cargo doc --no-deps
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Acknowledgments

- NEAT algorithm based on Stanley & Miikkulainen (2002)
- PAD model based on Mehrabian (1996)
- OCEAN model based on the Big Five personality theory

## Related Projects

- [BeeBotOS Core](../../crates/core) - Core types and utilities
- [BeeBotOS Memory](../../docs/memory-system.md) - Memory system documentation

---

For more information, see the [API Documentation](https://docs.rs/beebotos-brain) or the [main project documentation](../../docs).
