# PAD Emotional Model Specification

## Overview

The PAD (Pleasure-Arousal-Dominance) model is a dimensional framework for representing emotional states in three-dimensional space.

## Dimensions

### Pleasure (P)
- **Range**: -1.0 (unpleasant) to +1.0 (pleasant)
- **Description**: Represents the degree of pleasantness or unpleasantness
- **Examples**:
  - +1.0: Ecstatic, happy, delighted
  - 0.0: Neutral
  - -1.0: Miserable, unhappy, annoyed

### Arousal (A)
- **Range**: -1.0 (calm) to +1.0 (excited)
- **Description**: Represents the level of energy or activation
- **Examples**:
  - +1.0: Excited, alert, aroused
  - 0.0: Neutral
  - -1.0: Relaxed, calm, sleepy

### Dominance (D)
- **Range**: -1.0 (submissive) to +1.0 (dominant)
- **Description**: Represents the sense of control or influence
- **Examples**:
  - +1.0: Dominant, powerful, in-control
  - 0.0: Neutral
  - -1.0: Submissive, helpless, influenced

## Data Structure

```rust
/// PAD emotional state
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmotionState {
    pub pleasure: f64,   // P dimension
    pub arousal: f64,    // A dimension
    pub dominance: f64,  // D dimension
}

impl EmotionState {
    /// Create new emotional state
    pub fn new(pleasure: f64, arousal: f64, dominance: f64) -> Self {
        Self {
            pleasure: pleasure.clamp(-1.0, 1.0),
            arousal: arousal.clamp(-1.0, 1.0),
            dominance: dominance.clamp(-1.0, 1.0),
        }
    }

    /// Calculate distance between two emotional states
    pub fn distance(&self, other: &EmotionState) -> f64 {
        ((self.pleasure - other.pleasure).powi(2) +
         (self.arousal - other.arousal).powi(2) +
         (self.dominance - other.dominance).powi(2))
        .sqrt()
    }

    /// Linear interpolation between two states
    pub fn lerp(&self, target: &EmotionState, t: f64) -> EmotionState {
        EmotionState::new(
            self.pleasure + (target.pleasure - self.pleasure) * t,
            self.arousal + (target.arousal - self.arousal) * t,
            self.dominance + (target.dominance - self.dominance) * t,
        )
    }
}
```

## Predefined Emotional States

```rust
impl EmotionState {
    // Positive emotions
    pub const HAPPY: EmotionState = EmotionState::new(0.8, 0.4, 0.2);
    pub const EXCITED: EmotionState = EmotionState::new(0.6, 0.9, 0.3);
    pub const CONTENT: EmotionState = EmotionState::new(0.6, -0.2, 0.3);
    pub const PROUD: EmotionState = EmotionState::new(0.6, 0.3, 0.8);
    
    // Negative emotions
    pub const ANGRY: EmotionState = EmotionState::new(-0.6, 0.6, 0.6);
    pub const SAD: EmotionState = EmotionState::new(-0.8, -0.4, -0.4);
    pub const AFRAID: EmotionState = EmotionState::new(-0.7, 0.7, -0.6);
    pub const DISGUSTED: EmotionState = EmotionState::new(-0.6, 0.2, 0.1);
    
    // Neutral emotions
    pub const NEUTRAL: EmotionState = EmotionState::new(0.0, 0.0, 0.0);
    pub const SURPRISED: EmotionState = EmotionState::new(0.2, 0.9, 0.0);
    pub const BORED: EmotionState = EmotionState::new(-0.3, -0.6, -0.4);
}
```

## Dynamics

### Decay

Emotional states naturally decay toward neutral:

```rust
impl EmotionState {
    /// Apply decay toward neutral
    pub fn decay(&mut self, rate: f64) {
        let neutral = EmotionState::NEUTRAL;
        *self = self.lerp(&neutral, rate);
    }
}
```

### Stimulus Response

```rust
pub struct EmotionDynamics {
    current: EmotionState,
    decay_rate: f64,
    sensitivity: f64,
}

impl EmotionDynamics {
    /// Update emotion based on stimulus
    pub fn apply_stimulus(&mut self, stimulus: EmotionState, intensity: f64) {
        let adjusted_stimulus = EmotionState::new(
            stimulus.pleasure * intensity * self.sensitivity,
            stimulus.arousal * intensity * self.sensitivity,
            stimulus.dominance * intensity * self.sensitivity,
        );
        
        // Blend current emotion with stimulus
        self.current = self.current.lerp(&adjusted_stimulus, 0.3);
    }
    
    /// Update emotion over time
    pub fn update(&mut self, dt: f64) {
        self.current.decay(self.decay_rate * dt);
    }
}
```

### Contagion

Emotions can spread between agents:

```rust
/// Calculate emotional contagion
pub fn contagion(
    receiver: &mut EmotionState,
    sender: &EmotionState,
    receptivity: f64,
    intensity: f64,
) {
    // Only contagious if sender has high arousal
    if sender.arousal < 0.3 {
        return;
    }
    
    let transfer = EmotionState::new(
        sender.pleasure * receptivity * intensity,
        sender.arousal * receptivity * intensity * 0.5,
        sender.dominance * receptivity * intensity,
    );
    
    *receiver = receiver.lerp(&transfer, 0.2);
}
```

## Personality Influence

```rust
pub struct Personality {
    pub openness: f64,
    pub conscientiousness: f64,
    pub extraversion: f64,
    pub agreeableness: f64,
    pub neuroticism: f64,
}

impl Personality {
    /// Modify emotional response based on personality
    pub fn modulate_emotion(&self, base_emotion: EmotionState) -> EmotionState {
        EmotionState::new(
            // Extraverts experience more pleasure
            base_emotion.pleasure * (0.5 + 0.5 * self.extraversion),
            // Neurotic individuals have higher arousal
            base_emotion.arousal * (0.3 + 0.7 * self.neuroticism),
            // Openness affects dominance
            base_emotion.dominance * (0.5 + 0.5 * self.openness),
        )
    }
}
```

## Decision Making

```rust
/// Influence decision making based on emotion
pub fn emotional_decision_bias(emotion: &EmotionState) -> DecisionBias {
    DecisionBias {
        // Positive pleasure -> risk-seeking
        risk_tolerance: (emotion.pleasure + 1.0) / 2.0,
        
        // High arousal -> quick decisions
        deliberation_time: 1.0 - emotion.arousal.abs(),
        
        // High dominance -> assertive
        assertiveness: (emotion.dominance + 1.0) / 2.0,
        
        // Negative pleasure -> conservative
        conservatism: (-emotion.pleasure + 1.0) / 2.0,
    }
}
```

## Visualization

```rust
/// Convert to RGB color for visualization
pub fn to_color(&self) -> (u8, u8, u8) {
    // Pleasure -> Green/Red
    let r = if self.pleasure < 0.0 {
        (self.pleasure.abs() * 255.0) as u8
    } else {
        0
    };
    let g = if self.pleasure > 0.0 {
        (self.pleasure * 255.0) as u8
    } else {
        0
    };
    
    // Arousal -> Brightness
    let brightness = (self.arousal + 1.0) / 2.0;
    
    // Dominance -> Blue
    let b = ((self.dominance + 1.0) / 2.0 * 255.0 * brightness) as u8;
    
    (
        (r as f64 * brightness) as u8,
        (g as f64 * brightness) as u8,
        b,
    )
}
```

## References

- Mehrabian, A. (1996). Pleasure-arousal-dominance: A general framework for describing and measuring individual differences in temperament. Current Psychology, 14(4), 261-292.
