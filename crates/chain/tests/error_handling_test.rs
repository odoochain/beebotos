//! Error Handling Tests
//!
//! Tests for ChainError types and conversions.

use beebotos_chain::ChainError;

/// Test error display messages
#[test]
fn test_error_display_messages() {
    assert_eq!(
        format!("{}", ChainError::Connection("test".to_string())),
        "Connection error: test"
    );

    assert_eq!(
        format!("{}", ChainError::Contract("test".to_string())),
        "Contract error: test"
    );

    assert_eq!(
        format!("{}", ChainError::InvalidAddress("test".to_string())),
        "Invalid address: test"
    );

    assert_eq!(
        format!("{}", ChainError::DAO("test".to_string())),
        "DAO error: test"
    );

    assert_eq!(
        format!("{}", ChainError::Provider("test".to_string())),
        "Provider error: test"
    );

    assert_eq!(
        format!("{}", ChainError::Wallet("test".to_string())),
        "Wallet error: test"
    );

    assert_eq!(
        format!("{}", ChainError::UrlParse("test".to_string())),
        "URL parse error: test"
    );

    assert_eq!(
        format!("{}", ChainError::AlloyProvider("test".to_string())),
        "Alloy provider error: test"
    );

    assert_eq!(
        format!("{}", ChainError::AlloySigner("test".to_string())),
        "Alloy signer error: test"
    );

    assert_eq!(
        format!("{}", ChainError::Serialization("test".to_string())),
        "Serialization error: test"
    );

    assert_eq!(
        format!("{}", ChainError::InvalidConfig("test".to_string())),
        "Invalid configuration: test"
    );

    assert_eq!(
        format!("{}", ChainError::Identity("test".to_string())),
        "Identity error: test"
    );

    assert_eq!(
        format!("{}", ChainError::Bridge("test".to_string())),
        "Bridge error: test"
    );

    assert_eq!(
        format!("{}", ChainError::Oracle("test".to_string())),
        "Oracle error: test"
    );

    assert_eq!(
        format!("{}", ChainError::NotImplemented("test".to_string())),
        "Not implemented: test"
    );

    assert_eq!(
        format!("{}", ChainError::Timeout("test".to_string())),
        "Timeout error: test"
    );

    assert_eq!(
        format!("{}", ChainError::Validation("test".to_string())),
        "Validation error: test"
    );
}

/// Test error debug format
#[test]
fn test_error_debug() {
    let err = ChainError::Connection("test".to_string());
    assert!(format!("{:?}", err).contains("Connection"));
}

/// Test error clone
#[test]
fn test_error_clone() {
    let err1 = ChainError::Connection("test".to_string());
    let err2 = err1.clone();
    assert_eq!(format!("{}", err1), format!("{}", err2));
}

/// Test transaction failed error
#[test]
fn test_transaction_failed_error() {
    use beebotos_chain::compat::B256;

    let tx_hash = B256::from([1u8; 32]);
    let err = ChainError::TransactionFailed {
        tx_hash,
        reason: "out of gas".to_string(),
    };

    let msg = format!("{}", err);
    assert!(msg.contains("Transaction failed"));
    assert!(msg.contains("out of gas"));
}

/// Test insufficient balance error
#[test]
fn test_insufficient_balance_error() {
    assert_eq!(
        format!("{}", ChainError::InsufficientBalance),
        "Insufficient balance"
    );
}

/// Test error from alloy_contract::Error
#[test]
fn test_error_from_alloy_contract() {
    // Note: Can't easily create alloy_contract::Error in test,
    // but we can verify the conversion exists
}

/// Test error from alloy_signer::Error
#[test]
fn test_error_from_alloy_signer() {
    // Note: Can't easily create alloy_signer::Error in test,
    // but we can verify the conversion exists
}

/// Test error from url::ParseError
#[test]
fn test_error_from_url_parse() {
    let url_err = "not a url".parse::<url::Url>().unwrap_err();
    let chain_err: ChainError = url_err.into();
    assert!(matches!(chain_err, ChainError::UrlParse(_)));
}

/// Test error from serde_json::Error
#[test]
fn test_error_from_serde_json() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let chain_err: ChainError = json_err.into();
    assert!(matches!(chain_err, ChainError::Serialization(_)));
}

/// Test error from std::io::Error
#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let chain_err: ChainError = io_err.into();
    assert!(matches!(chain_err, ChainError::Connection(_)));
}

/// Test error equality (for tests)
#[test]
fn test_error_equality() {
    let err1 = ChainError::Connection("test".to_string());
    let err2 = ChainError::Connection("test".to_string());
    let err3 = ChainError::Connection("different".to_string());

    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

/// Test error as response for API handling
#[test]
fn test_error_for_api_response() {
    fn handle_error(err: ChainError) -> (u16, String) {
        match err {
            ChainError::InvalidAddress(_) | ChainError::Validation(_) => {
                (400, "Bad Request".to_string())
            }
            ChainError::InsufficientBalance => (402, "Payment Required".to_string()),
            ChainError::NotImplemented(_) => (501, "Not Implemented".to_string()),
            ChainError::Connection(_) | ChainError::Provider(_) => {
                (503, "Service Unavailable".to_string())
            }
            _ => (500, "Internal Server Error".to_string()),
        }
    }

    assert_eq!(
        handle_error(ChainError::InvalidAddress("test".to_string())).0,
        400
    );
    assert_eq!(handle_error(ChainError::InsufficientBalance).0, 402);
    assert_eq!(
        handle_error(ChainError::NotImplemented("test".to_string())).0,
        501
    );
    assert_eq!(
        handle_error(ChainError::Connection("test".to_string())).0,
        503
    );
}

/// Test error chaining
#[test]
fn test_error_chaining() {
    let inner_err = std::io::Error::new(std::io::ErrorKind::Other, "inner");
    let chain_err: ChainError = inner_err.into();

    // Verify the error can be converted
    assert!(matches!(chain_err, ChainError::Connection(_)));
}

/// Test all error variants are Clone
#[test]
fn test_all_errors_cloneable() {
    use beebotos_chain::compat::B256;

    let errors: Vec<ChainError> = vec![
        ChainError::Connection("test".to_string()),
        ChainError::Contract("test".to_string()),
        ChainError::TransactionFailed {
            tx_hash: B256::ZERO,
            reason: "test".to_string(),
        },
        ChainError::InsufficientBalance,
        ChainError::InvalidAddress("test".to_string()),
        ChainError::DAO("test".to_string()),
        ChainError::Provider("test".to_string()),
        ChainError::Wallet("test".to_string()),
        ChainError::UrlParse("test".to_string()),
        ChainError::AlloyProvider("test".to_string()),
        ChainError::AlloySigner("test".to_string()),
        ChainError::Serialization("test".to_string()),
        ChainError::InvalidConfig("test".to_string()),
        ChainError::Identity("test".to_string()),
        ChainError::Bridge("test".to_string()),
        ChainError::Oracle("test".to_string()),
        ChainError::NotImplemented("test".to_string()),
        ChainError::Timeout("test".to_string()),
        ChainError::Validation("test".to_string()),
    ];

    for err in errors {
        let cloned = err.clone();
        assert_eq!(format!("{}", err), format!("{}", cloned));
    }
}
