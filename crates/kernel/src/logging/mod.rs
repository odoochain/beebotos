//! Logging Module
//!
//! Provides structured logging, audit trails, and metrics collection.
//!
//! ## Submodules
//!
//! - `structured`: Structured log output
//! - `audit`: Security audit logging
//! - `metrics`: Metrics collection and export

pub mod structured;
pub mod audit;
pub mod metrics;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Log severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level for detailed debugging
    Trace,
    /// Debug level for development
    Debug,
    /// Info level for general messages
    Info,
    /// Warn level for potential issues
    Warn,
    /// Error level for failures
    Error,
    /// Fatal level for critical errors
    Fatal,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }

    pub fn numeric_value(&self) -> u8 {
        match self {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
            LogLevel::Fatal => 5,
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unix timestamp of the log entry
    pub timestamp: u64,
    /// Log severity level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Source component/module
    pub source: String,
    /// Distributed trace ID
    pub trace_id: Option<String>,
    /// Span ID for tracing
    pub span_id: Option<String>,
    /// Parent span ID for nested spans
    pub parent_span_id: Option<String>,
    /// Additional structured fields
    pub fields: HashMap<String, serde_json::Value>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            level,
            message: message.into(),
            source: source.into(),
            trace_id: None,
            span_id: None,
            parent_span_id: None,
            fields: HashMap::new(),
        }
    }

    pub fn with_trace(mut self, trace_id: impl Into<String>, span_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self.span_id = Some(span_id.into());
        self
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }
}

pub trait LogSink: Send + Sync {
    fn write(&self, entry: &LogEntry) -> Result<(), LogError>;
    fn flush(&self) -> Result<(), LogError>;
}

/// Log errors
#[derive(Debug, Clone)]
pub enum LogError {
    /// IO error
    IoError(String),
    /// Serialization error
    SerializationError(String),
    /// Sink is full
    SinkFull,
    /// Entry filtered
    Filtered,
}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::IoError(e) => write!(f, "IO error: {}", e),
            LogError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            LogError::SinkFull => write!(f, "Log sink is full"),
            LogError::Filtered => write!(f, "Log entry filtered"),
        }
    }
}

impl std::error::Error for LogError {}

/// Logger
pub struct Logger {
    /// Minimum log level
    min_level: LogLevel,
    /// Log sinks
    sinks: Vec<Box<dyn LogSink>>,
    /// Log filters
    filters: Vec<Box<dyn Fn(&LogEntry) -> bool + Send + Sync>>,
}

impl Logger {
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            min_level,
            sinks: vec![],
            filters: vec![],
        }
    }

    pub fn add_sink(&mut self, sink: Box<dyn LogSink>) {
        self.sinks.push(sink);
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&LogEntry) -> bool + Send + Sync + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn log(&self, entry: LogEntry) {
        if entry.level.numeric_value() < self.min_level.numeric_value() {
            return;
        }

        for filter in &self.filters {
            if !filter(&entry) {
                return;
            }
        }

        for sink in &self.sinks {
            let _ = sink.write(&entry);
        }
    }

    pub fn trace(&self, message: impl Into<String>, source: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Trace, message, source));
    }

    pub fn debug(&self, message: impl Into<String>, source: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Debug, message, source));
    }

    pub fn info(&self, message: impl Into<String>, source: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Info, message, source));
    }

    pub fn warn(&self, message: impl Into<String>, source: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Warn, message, source));
    }

    pub fn error(&self, message: impl Into<String>, source: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Error, message, source));
    }

    pub fn fatal(&self, message: impl Into<String>, source: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Fatal, message, source));
    }

    pub fn flush(&self) {
        for sink in &self.sinks {
            let _ = sink.flush();
        }
    }
}

/// Console log sink
pub struct ConsoleSink {
    /// Pretty print mode
    pretty_print: bool,
}

impl ConsoleSink {
    pub fn new(pretty_print: bool) -> Self {
        Self { pretty_print }
    }
}

impl LogSink for ConsoleSink {
    fn write(&self, entry: &LogEntry) -> Result<(), LogError> {
        if self.pretty_print {
            println!(
                "[{}] {} - {}: {}",
                entry.timestamp,
                entry.level.as_str(),
                entry.source,
                entry.message
            );
        } else {
            println!("{}", serde_json::to_string(entry).map_err(|e| LogError::SerializationError(e.to_string()))?);
        }
        Ok(())
    }

    fn flush(&self) -> Result<(), LogError> {
        Ok(())
    }
}

/// File log sink
pub struct FileSink {
    /// Log file path
    path: std::path::PathBuf,
    /// Maximum file size
    max_size: u64,
    /// Maximum number of files
    max_files: usize,
}

impl FileSink {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: path.into(),
            max_size: 100 * 1024 * 1024, // 100MB
            max_files: 5,
        }
    }

    fn rotate_if_needed(&self) -> Result<(), LogError> {
        if !self.path.exists() {
            return Ok(());
        }

        let metadata = std::fs::metadata(&self.path).map_err(|e| LogError::IoError(e.to_string()))?;
        
        if metadata.len() >= self.max_size {
            // Rotate files
            for i in (1..self.max_files).rev() {
                let old_path = self.path.with_extension(format!(".{}", i));
                let new_path = self.path.with_extension(format!(".{}", i + 1));
                
                if old_path.exists() {
                    std::fs::rename(&old_path, &new_path).map_err(|e| LogError::IoError(e.to_string()))?;
                }
            }

            std::fs::rename(&self.path, self.path.with_extension(".1"))
                .map_err(|e| LogError::IoError(e.to_string()))?;
        }

        Ok(())
    }
}

impl LogSink for FileSink {
    fn write(&self, entry: &LogEntry) -> Result<(), LogError> {
        self.rotate_if_needed()?;

        let line = serde_json::to_string(entry).map_err(|e| LogError::SerializationError(e.to_string()))?;
        
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| LogError::IoError(e.to_string()))?;

        writeln!(file, "{}", line).map_err(|e| LogError::IoError(e.to_string()))?;
        
        Ok(())
    }

    fn flush(&self) -> Result<(), LogError> {
        Ok(())
    }
}

/// Ring buffer log sink
pub struct RingBufferSink {
    /// Log buffer
    buffer: std::sync::Mutex<Vec<LogEntry>>,
    /// Buffer capacity
    capacity: usize,
}

impl RingBufferSink {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: std::sync::Mutex::new(Vec::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.buffer.lock().unwrap().clone()
    }
}

impl LogSink for RingBufferSink {
    fn write(&self, entry: &LogEntry) -> Result<(), LogError> {
        let mut buffer = self.buffer.lock().unwrap();
        
        if buffer.len() >= self.capacity {
            buffer.remove(0);
        }
        
        buffer.push(entry.clone());
        Ok(())
    }

    fn flush(&self) -> Result<(), LogError> {
        Ok(())
    }
}
