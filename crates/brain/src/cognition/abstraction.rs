//! Abstraction

/// Abstraction levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbstractionLevel {
    Concrete,
    Specific,
    General,
    Abstract,
    Meta,
}

/// Abstract concept
#[derive(Debug)]
pub struct AbstractConcept {
    pub name: String,
    pub level: AbstractionLevel,
    pub instances: Vec<String>,
}

impl AbstractConcept {
    pub fn new(name: impl Into<String>, level: AbstractionLevel) -> Self {
        Self {
            name: name.into(),
            level,
            instances: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, instance: impl Into<String>) {
        self.instances.push(instance.into());
    }
}
