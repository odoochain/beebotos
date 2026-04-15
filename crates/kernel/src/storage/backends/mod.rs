//! Storage Backends
//!
//! Provides various storage backend implementations:
//! - Memory: In-memory volatile storage
//! - RocksDB: High-performance persistent storage (optional)
//! - Redb: Pure Rust ACID-compliant storage (optional)
//! - SQLite: ACID-compliant SQL database (optional)
//! - Filesystem: Simple file-based storage
//! - Encrypted: Encryption wrapper for any backend

pub mod encrypted;
pub mod filesystem;
pub mod memory;
pub mod redb;
pub mod rocksdb;
pub mod sqlite;

/// Backend type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// In-memory storage
    Memory,
    /// RocksDB backend
    RocksDb,
    /// Redb backend
    Redb,
    /// SQLite backend
    Sqlite,
    /// Filesystem backend
    Filesystem,
    /// Encrypted wrapper
    Encrypted,
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendType::Memory => write!(f, "memory"),
            BackendType::RocksDb => write!(f, "rocksdb"),
            BackendType::Redb => write!(f, "redb"),
            BackendType::Sqlite => write!(f, "sqlite"),
            BackendType::Filesystem => write!(f, "filesystem"),
            BackendType::Encrypted => write!(f, "encrypted"),
        }
    }
}

/// Macro to generate stub backend implementation when feature is disabled
///
/// This macro eliminates the repetitive boilerplate for disabled features.
/// Each backend that requires a feature flag should use this macro.
///
/// # Example
/// ```ignore
/// define_stub_backend!(RocksDbStorage, "rocksdb");
/// ```
#[macro_export]
macro_rules! define_stub_backend {
    ($name:ident, $feature:literal) => {
        /// Stub implementation when feature is disabled
        #[cfg(not(feature = $feature))]
        #[derive(Debug)]
        pub struct $name;

        #[cfg(not(feature = $feature))]
        impl $name {
            /// Stub open method (feature disabled)
            pub fn open<P: AsRef<std::path::Path>>(
                _path: P,
            ) -> std::result::Result<Self, $crate::storage::StorageError> {
                Err($crate::storage::StorageError::IoError(
                    concat!($feature, " feature not enabled").to_string(),
                ))
            }
        }

        #[cfg(not(feature = $feature))]
        impl $crate::storage::StorageBackend for $name {
            fn put(
                &self,
                _key: &str,
                _data: &[u8],
                _metadata: $crate::storage::EntryMetadata,
            ) -> std::result::Result<(), $crate::storage::StorageError> {
                Err($crate::storage::StorageError::IoError(
                    concat!($feature, " feature not enabled").to_string(),
                ))
            }

            fn get(
                &self,
                _key: &str,
            ) -> std::result::Result<
                Option<$crate::storage::StorageEntry>,
                $crate::storage::StorageError,
            > {
                Err($crate::storage::StorageError::IoError(
                    concat!($feature, " feature not enabled").to_string(),
                ))
            }

            fn delete(&self, _key: &str) -> std::result::Result<(), $crate::storage::StorageError> {
                Err($crate::storage::StorageError::IoError(
                    concat!($feature, " feature not enabled").to_string(),
                ))
            }

            fn list(
                &self,
                _prefix: &str,
            ) -> std::result::Result<Vec<String>, $crate::storage::StorageError> {
                Err($crate::storage::StorageError::IoError(
                    concat!($feature, " feature not enabled").to_string(),
                ))
            }
        }
    };
}

/// Macro to generate stub implementation for encrypted wrapper
///
/// Special case for EncryptedStorage which has generic parameter
#[macro_export]
macro_rules! define_stub_encrypted_backend {
    () => {
        #[cfg(not(feature = "encryption"))]
        impl<B: $crate::storage::StorageBackend> EncryptedStorage<B> {
            #[allow(dead_code)]
            fn encrypt(
                &self,
                _plaintext: &[u8],
            ) -> std::result::Result<Vec<u8>, $crate::storage::StorageError> {
                Err($crate::storage::StorageError::IoError(
                    "encryption feature not enabled".to_string(),
                ))
            }

            #[allow(dead_code)]
            fn decrypt(
                &self,
                _ciphertext: &[u8],
            ) -> std::result::Result<Vec<u8>, $crate::storage::StorageError> {
                Err($crate::storage::StorageError::IoError(
                    "encryption feature not enabled".to_string(),
                ))
            }
        }

        #[cfg(not(feature = "encryption"))]
        impl<B: $crate::storage::StorageBackend> $crate::storage::StorageBackend
            for EncryptedStorage<B>
        {
            fn put(
                &self,
                _key: &str,
                _data: &[u8],
                _metadata: $crate::storage::EntryMetadata,
            ) -> std::result::Result<(), $crate::storage::StorageError> {
                Err($crate::storage::StorageError::IoError(
                    "encryption feature not enabled".to_string(),
                ))
            }

            fn get(
                &self,
                _key: &str,
            ) -> std::result::Result<
                Option<$crate::storage::StorageEntry>,
                $crate::storage::StorageError,
            > {
                Err($crate::storage::StorageError::IoError(
                    "encryption feature not enabled".to_string(),
                ))
            }

            fn delete(&self, key: &str) -> std::result::Result<(), $crate::storage::StorageError> {
                self.inner.delete(key)
            }

            fn list(
                &self,
                prefix: &str,
            ) -> std::result::Result<Vec<String>, $crate::storage::StorageError> {
                self.inner.list(prefix)
            }

            fn exists(
                &self,
                key: &str,
            ) -> std::result::Result<bool, $crate::storage::StorageError> {
                self.inner.exists(key)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_display() {
        assert_eq!(format!("{}", BackendType::Memory), "memory");
        assert_eq!(format!("{}", BackendType::RocksDb), "rocksdb");
        assert_eq!(format!("{}", BackendType::Sqlite), "sqlite");
        assert_eq!(format!("{}", BackendType::Redb), "redb");
        assert_eq!(format!("{}", BackendType::Filesystem), "filesystem");
        assert_eq!(format!("{}", BackendType::Encrypted), "encrypted");
    }
}
