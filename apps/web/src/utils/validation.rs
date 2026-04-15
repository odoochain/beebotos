//! Form validation utilities

use std::fmt::Display;

/// Validation error
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Validation result
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// String validators
pub struct StringValidators;

impl StringValidators {
    /// Required field
    pub fn required(field: &str, value: &str) -> ValidationResult {
        if value.trim().is_empty() {
            Err(vec![ValidationError::new(field, "This field is required")])
        } else {
            Ok(())
        }
    }

    /// Minimum length
    pub fn min_length(field: &str, value: &str, min: usize) -> ValidationResult {
        if value.len() < min {
            Err(vec![ValidationError::new(
                field,
                format!("Must be at least {} characters", min),
            )])
        } else {
            Ok(())
        }
    }

    /// Maximum length
    pub fn max_length(field: &str, value: &str, max: usize) -> ValidationResult {
        if value.len() > max {
            Err(vec![ValidationError::new(
                field,
                format!("Must be no more than {} characters", max),
            )])
        } else {
            Ok(())
        }
    }

    /// Email format (simplified)
    pub fn email(field: &str, value: &str) -> ValidationResult {
        if !value.contains('@') || !value.contains('.') {
            Err(vec![ValidationError::new(field, "Invalid email format")])
        } else {
            Ok(())
        }
    }

    /// URL format
    pub fn url(field: &str, value: &str) -> ValidationResult {
        if !value.starts_with("http://") && !value.starts_with("https://") {
            Err(vec![ValidationError::new(
                field,
                "Must start with http:// or https://",
            )])
        } else {
            Ok(())
        }
    }

    /// Ethereum address
    pub fn ethereum_address(field: &str, value: &str) -> ValidationResult {
        if !value.starts_with("0x") || value.len() != 42 {
            Err(vec![ValidationError::new(
                field,
                "Invalid Ethereum address",
            )])
        } else {
            Ok(())
        }
    }

    /// Alphanumeric only
    pub fn alphanumeric(field: &str, value: &str) -> ValidationResult {
        if !value.chars().all(|c| c.is_alphanumeric() || c == '_') {
            Err(vec![ValidationError::new(
                field,
                "Only letters, numbers, and underscores allowed",
            )])
        } else {
            Ok(())
        }
    }
}

/// Numeric validators
pub struct NumericValidators;

impl NumericValidators {
    /// Greater than
    pub fn greater_than<T: PartialOrd + Display>(
        field: &str,
        value: T,
        min: T,
    ) -> ValidationResult {
        if value <= min {
            Err(vec![ValidationError::new(
                field,
                format!("Must be greater than {}", min),
            )])
        } else {
            Ok(())
        }
    }

    /// Less than
    pub fn less_than<T: PartialOrd + Display>(field: &str, value: T, max: T) -> ValidationResult {
        if value >= max {
            Err(vec![ValidationError::new(
                field,
                format!("Must be less than {}", max),
            )])
        } else {
            Ok(())
        }
    }

    /// Range inclusive
    pub fn range<T: PartialOrd + Display>(
        field: &str,
        value: T,
        min: T,
        max: T,
    ) -> ValidationResult {
        if value < min || value > max {
            Err(vec![ValidationError::new(
                field,
                format!("Must be between {} and {}", min, max),
            )])
        } else {
            Ok(())
        }
    }
}

/// Collection validators
pub struct CollectionValidators;

impl CollectionValidators {
    /// Not empty
    pub fn not_empty<T>(field: &str, value: &[T]) -> ValidationResult {
        if value.is_empty() {
            Err(vec![ValidationError::new(
                field,
                "At least one item required",
            )])
        } else {
            Ok(())
        }
    }

    /// Minimum items
    pub fn min_items<T>(field: &str, value: &[T], min: usize) -> ValidationResult {
        if value.len() < min {
            Err(vec![ValidationError::new(
                field,
                format!("At least {} items required", min),
            )])
        } else {
            Ok(())
        }
    }

    /// Maximum items
    pub fn max_items<T>(field: &str, value: &[T], max: usize) -> ValidationResult {
        if value.len() > max {
            Err(vec![ValidationError::new(
                field,
                format!("No more than {} items allowed", max),
            )])
        } else {
            Ok(())
        }
    }
}

/// Combine multiple validations
pub fn combine(results: Vec<ValidationResult>) -> ValidationResult {
    let errors: Vec<_> = results
        .into_iter()
        .filter_map(|r| r.err())
        .flatten()
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validation builder for forms
#[derive(Default, Clone)]
pub struct FormValidator {
    errors: Vec<ValidationError>,
}

impl FormValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(&mut self, result: ValidationResult) -> &mut Self {
        if let Err(errors) = result {
            self.errors.extend(errors);
        }
        self
    }

    pub fn result(&self) -> ValidationResult {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    pub fn errors_for_field(&self, field: &str) -> Vec<&ValidationError> {
        self.errors.iter().filter(|e| e.field == field).collect()
    }

    pub fn has_error(&self, field: &str) -> bool {
        self.errors.iter().any(|e| e.field == field)
    }

    pub fn first_error(&self, field: &str) -> Option<&ValidationError> {
        self.errors.iter().find(|e| e.field == field)
    }

    pub fn first_error_message(&self, field: &str) -> Option<String> {
        self.first_error(field).map(|e| e.message.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required() {
        assert!(StringValidators::required("name", "test").is_ok());
        assert!(StringValidators::required("name", "").is_err());
        assert!(StringValidators::required("name", "   ").is_err());
    }

    #[test]
    fn test_min_length() {
        assert!(StringValidators::min_length("name", "test", 3).is_ok());
        assert!(StringValidators::min_length("name", "te", 3).is_err());
    }

    #[test]
    fn test_ethereum_address() {
        assert!(StringValidators::ethereum_address(
            "addr",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1"
        )
        .is_ok());
        assert!(StringValidators::ethereum_address("addr", "invalid").is_err());
    }

    #[test]
    fn test_email() {
        assert!(StringValidators::email("email", "test@example.com").is_ok());
        assert!(StringValidators::email("email", "invalid").is_err());
    }

    #[test]
    fn test_form_validator() {
        let mut validator = FormValidator::new();
        validator
            .validate(StringValidators::required("name", "test"))
            .validate(StringValidators::min_length("name", "test", 3));

        assert!(validator.is_valid());

        let mut validator = FormValidator::new();
        validator
            .validate(StringValidators::required("name", ""))
            .validate(StringValidators::min_length("name", "te", 3));

        assert!(!validator.is_valid());
        assert_eq!(validator.errors().len(), 2);
    }

    #[test]
    fn test_combine() {
        let results = vec![
            StringValidators::required("name", "test"),
            StringValidators::min_length("name", "test", 3),
        ];
        assert!(combine(results).is_ok());

        let results = vec![
            StringValidators::required("name", ""),
            StringValidators::min_length("name", "test", 3),
        ];
        assert!(combine(results).is_err());
    }
}
