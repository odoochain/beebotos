//! Communication Router
//!
//! Routes messages between different platforms and agents.

use regex::Regex;

use super::{Message, PlatformType};

/// Route configuration
#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub from_platform: PlatformType,
    pub to_platform: PlatformType,
    pub filter: Option<Regex>,
    pub transform: Option<MessageTransform>,
}

/// Message transformation
#[derive(Debug, Clone)]
pub enum MessageTransform {
    Prepend(String),
    Append(String),
    Replace {
        pattern: String,
        replacement: String,
    },
}

/// Communication router
pub struct CommunicationRouter {
    routes: Vec<RouteConfig>,
    default_route: Option<PlatformType>,
}

impl CommunicationRouter {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            default_route: None,
        }
    }

    /// Add a route
    pub fn add_route(&mut self, config: RouteConfig) {
        self.routes.push(config);
    }

    /// Set default route
    pub fn set_default_route(&mut self, platform: PlatformType) {
        self.default_route = Some(platform);
    }

    /// Route a message
    pub fn route(&self, message: &Message) -> Vec<PlatformType> {
        let mut targets = Vec::new();

        for route in &self.routes {
            if route.from_platform == message.platform {
                // Check filter
                if let Some(ref filter) = route.filter {
                    if !filter.is_match(&message.content) {
                        continue;
                    }
                }
                targets.push(route.to_platform);
            }
        }

        // Use default if no routes matched
        if targets.is_empty() {
            if let Some(default) = self.default_route {
                targets.push(default);
            }
        }

        targets
    }

    /// Apply transformation to message
    pub fn transform(&self, message: &mut Message, transform: &MessageTransform) {
        match transform {
            MessageTransform::Prepend(prefix) => {
                message.content = format!("{}{}", prefix, message.content);
            }
            MessageTransform::Append(suffix) => {
                message.content = format!("{}{}", message.content, suffix);
            }
            MessageTransform::Replace {
                pattern,
                replacement,
            } => {
                message.content = message.content.replace(pattern, replacement);
            }
        }
    }

    /// Get routes for a platform
    pub fn get_routes(&self, from: PlatformType) -> Vec<&RouteConfig> {
        self.routes
            .iter()
            .filter(|r| r.from_platform == from)
            .collect()
    }
}

impl Default for CommunicationRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn create_test_message(platform: PlatformType, content: &str) -> Message {
        Message::new(Uuid::new_v4(), platform, content)
    }

    #[test]
    fn test_route_matching() {
        let mut router = CommunicationRouter::new();
        router.add_route(RouteConfig {
            from_platform: PlatformType::Slack,
            to_platform: PlatformType::Discord,
            filter: None,
            transform: None,
        });

        let msg = create_test_message(PlatformType::Slack, "Hello");
        let targets = router.route(&msg);
        assert_eq!(targets, vec![PlatformType::Discord]);
    }
}
