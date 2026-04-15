//! DID Operations

/// DID method types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DIDMethod {
    Ethr,
    Key,
    Web,
    Ion,
}

impl DIDMethod {
    pub fn prefix(&self) -> &'static str {
        match self {
            DIDMethod::Ethr => "did:ethr:",
            DIDMethod::Key => "did:key:",
            DIDMethod::Web => "did:web:",
            DIDMethod::Ion => "did:ion:",
        }
    }
}

/// DID parser
pub struct DIDParser;

impl DIDParser {
    pub fn parse(did: &str) -> anyhow::Result<(DIDMethod, String)> {
        if let Some(rest) = did.strip_prefix("did:ethr:") {
            Ok((DIDMethod::Ethr, rest.to_string()))
        } else if let Some(rest) = did.strip_prefix("did:key:") {
            Ok((DIDMethod::Key, rest.to_string()))
        } else if let Some(rest) = did.strip_prefix("did:web:") {
            Ok((DIDMethod::Web, rest.to_string()))
        } else if let Some(rest) = did.strip_prefix("did:ion:") {
            Ok((DIDMethod::Ion, rest.to_string()))
        } else {
            return Err(anyhow::anyhow!("Unknown DID method: {}", did));
        }
    }

    pub fn create(method: DIDMethod, identifier: &str) -> String {
        format!("{}{}", method.prefix(), identifier)
    }
}
