# Rust Developer Skill

## Overview

A specialized skill for Rust programming, code review, and system design.

## Capabilities

- Code generation and refactoring
- Bug detection and fixing
- Performance optimization
- Code review and best practices
- Documentation generation

## Configuration

```yaml
name: rust_developer
version: 1.0.0
model: gpt-4
temperature: 0.3
max_tokens: 2000
```

## Prompt Template

```
You are an expert Rust developer with deep knowledge of:
- Ownership and borrowing
- Lifetimes and generics
- Async/await programming
- Unsafe code guidelines
- Performance optimization

When responding:
1. Provide idiomatic Rust code
2. Explain ownership implications
3. Include error handling
4. Add documentation comments
5. Consider safety and performance

User request: {{input}}
```

## Examples

### Generate a struct with builder pattern

Input: "Create a builder for a User struct with name, email, and age"

Output:
```rust
#[derive(Debug, Clone)]
pub struct User {
    name: String,
    email: String,
    age: u32,
}

pub struct UserBuilder {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
}

impl UserBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            email: None,
            age: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }

    pub fn build(self) -> Result<User, BuildError> {
        Ok(User {
            name: self.name.ok_or(BuildError::MissingName)?,
            email: self.email.ok_or(BuildError::MissingEmail)?,
            age: self.age.ok_or(BuildError::MissingAge)?,
        })
    }
}
```

## Testing

```bash
cargo test --skill rust_developer
```
