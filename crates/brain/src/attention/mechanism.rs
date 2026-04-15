//! Attention mechanism for focus and saliency

use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Attention-weighted item
#[derive(Debug, Clone)]
pub struct AttentionItem<T> {
    pub content: T,
    pub saliency: f32,
    pub last_accessed: u64,
}

impl<T> PartialEq for AttentionItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.saliency == other.saliency
    }
}

impl<T> Eq for AttentionItem<T> {}

impl<T> PartialOrd for AttentionItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.saliency.partial_cmp(&other.saliency)
    }
}

impl<T> Ord for AttentionItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Attention spotlight controller
pub struct AttentionSpotlight<T> {
    items: Vec<AttentionItem<T>>,
    focus_capacity: usize,
    current_focus: Vec<usize>,
    decay_rate: f32,
}

impl<T> AttentionSpotlight<T> {
    pub fn new(focus_capacity: usize) -> Self {
        Self {
            items: Vec::new(),
            focus_capacity,
            current_focus: Vec::new(),
            decay_rate: 0.95,
        }
    }
    
    /// Add item to attention space
    pub fn add_item(&mut self, content: T, initial_saliency: f32) {
        self.items.push(AttentionItem {
            content,
            saliency: initial_saliency,
            last_accessed: crate::utils::current_timestamp_secs(),
        });
    }
    
    /// Update attention based on current context
    pub fn update_attention(&mut self, context: &Context) {
        // Decay all saliency
        for item in &mut self.items {
            item.saliency *= self.decay_rate;
        }
        
        // Boost items matching context
        for (i, item) in self.items.iter_mut().enumerate() {
            let relevance = calculate_relevance(&item.content, context);
            item.saliency += relevance * 0.3;
            item.saliency = item.saliency.min(1.0);
            
            if relevance > 0.5 {
                item.last_accessed = crate::utils::current_timestamp_secs();
            }
        }
        
        // Update focus
        self.update_focus();
    }
    
    fn update_focus(&mut self) {
        // Select top-K items by saliency
        let mut indexed: Vec<_> = self.items.iter()
            .enumerate()
            .map(|(i, item)| (i, item.saliency))
            .collect();
        
        indexed.sort_by(|a, b| crate::utils::compare_f32(&b.1, &a.1));
        
        self.current_focus = indexed.iter()
            .take(self.focus_capacity)
            .map(|(i, _)| *i)
            .collect();
    }
    
    /// Get items in current focus
    pub fn get_focused_items(&self) -> Vec<&T> {
        self.current_focus.iter()
            .filter_map(|&i| self.items.get(i).map(|item| &item.content))
            .collect()
    }
    
    /// Direct attention to specific item
    pub fn focus_on(&mut self, index: usize) {
        if index < self.items.len() {
            self.items[index].saliency = 1.0;
            self.items[index].last_accessed = crate::utils::current_timestamp_secs();
            
            if !self.current_focus.contains(&index) {
                if self.current_focus.len() >= self.focus_capacity {
                    self.current_focus.pop();
                }
                self.current_focus.insert(0, index);
            }
        }
    }
    
    /// Get attention statistics
    pub fn stats(&self) -> AttentionStats {
        let avg_saliency = if self.items.is_empty() {
            0.0
        } else {
            self.items.iter().map(|i| i.saliency).sum::<f32>() / self.items.len() as f32
        };
        
        AttentionStats {
            total_items: self.items.len(),
            focused_items: self.current_focus.len(),
            average_saliency: avg_saliency,
            focus_capacity: self.focus_capacity,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub keywords: Vec<String>,
    pub emotional_state: f32,
    pub urgency: f32,
}

fn calculate_relevance<T>(_content: &T, _context: &Context) -> f32 {
    // Placeholder - would use actual content analysis
    rand::random::<f32>() * 0.5
}

// 使用 utils::current_timestamp_secs() 代替局部函数

#[derive(Debug, Clone)]
pub struct AttentionStats {
    pub total_items: usize,
    pub focused_items: usize,
    pub average_saliency: f32,
    pub focus_capacity: usize,
}
