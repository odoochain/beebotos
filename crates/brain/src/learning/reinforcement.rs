//! Reinforcement learning module

use std::collections::HashMap;

/// Q-learning agent
pub struct QLearningAgent {
    q_table: HashMap<(String, String), f64>,
    learning_rate: f64,
    discount_factor: f64,
    epsilon: f64,
}

impl QLearningAgent {
    pub fn new() -> Self {
        Self {
            q_table: HashMap::new(),
            learning_rate: 0.1,
            discount_factor: 0.9,
            epsilon: 0.1,
        }
    }
    
    pub fn choose_action(&self, state: &str, actions: &[String]) -> String {
        if actions.is_empty() {
            return "noop".to_string();
        }
        
        if rand::random::<f64>() < self.epsilon {
            actions[rand::random::<usize>() % actions.len()].clone()
        } else {
            actions.iter()
                .max_by(|a, b| {
                    let qa = self.q_table.get(&(state.to_string(), (*a).clone())).unwrap_or(&0.0);
                    let qb = self.q_table.get(&(state.to_string(), (*b).clone())).unwrap_or(&0.0);
                    crate::utils::compare_f32(qa, qb)
                })
                .cloned()
                .unwrap_or_else(|| actions[0].clone())
        }
    }
    
    pub fn learn(&mut self, state: &str, action: &str, reward: f64, next_state: &str) {
        let current_q = *self.q_table.get(&(state.to_string(), action.to_string())).unwrap_or(&0.0);
        let max_next_q = self.q_table.iter()
            .filter(|((s, _), _)| s == next_state)
            .map(|(_, v)| *v)
            .fold(0.0, f64::max);
        
        let new_q = current_q + self.learning_rate * (reward + self.discount_factor * max_next_q - current_q);
        self.q_table.insert((state.to_string(), action.to_string()), new_q);
    }
}
