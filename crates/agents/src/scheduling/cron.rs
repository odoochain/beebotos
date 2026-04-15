//! Cron Scheduler
//!
//! Three scheduling modes:
//! - OneShot (ISO 8601 timestamp)
//! - Interval (milliseconds)
//! - Cron expression (5-field with IANA timezone)

use std::collections::HashMap;

use chrono::{DateTime, Utc};
// use cron::Schedule; // cron crate Schedule - temporarily disabled
use serde::{Deserialize, Serialize};

use crate::session::SessionKey;
// use std::str::FromStr;
// use std::time::Duration;

/// Cron scheduler
pub struct CronScheduler {
    jobs: HashMap<JobId, CronJob>,
    persistence: Option<Box<dyn CronPersistence>>,
}

/// Job ID
pub type JobId = String;

/// Cron job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: JobId,
    pub session: SessionKey,
    pub schedule: ScheduleType,
    pub command: String,
    pub context_mode: ContextMode,
    pub last_run: Option<u64>,
    pub next_run: Option<u64>,
    pub run_count: u64,
    pub max_runs: Option<u64>,
}

/// Schedule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    /// One-time execution at specific time
    OneShot(DateTime<Utc>),
    /// Fixed interval in milliseconds
    Interval(u64),
    /// Cron expression (expression, timezone)
    Cron(String, String),
}

/// Context execution mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ContextMode {
    /// Share main session context
    MainSession,
    /// Run in isolated session
    Isolated,
}

/// Persistence interface
#[async_trait::async_trait]
pub trait CronPersistence: Send + Sync {
    async fn save(&self, jobs: &HashMap<JobId, CronJob>) -> Result<(), CronError>;
    async fn load(&self) -> Result<HashMap<JobId, CronJob>, CronError>;
}

impl CronScheduler {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            persistence: None,
        }
    }

    pub fn with_persistence(mut self, persistence: Box<dyn CronPersistence>) -> Self {
        self.persistence = Some(persistence);
        self
    }

    pub fn add_job(&mut self, job: CronJob) -> JobId {
        let id = job.id.clone();
        self.jobs.insert(id.clone(), job);
        id
    }

    pub fn remove_job(&mut self, id: &JobId) -> Option<CronJob> {
        self.jobs.remove(id)
    }

    pub fn get_job(&self, id: &JobId) -> Option<&CronJob> {
        self.jobs.get(id)
    }

    /// Run due jobs and return executed jobs
    pub async fn run_due_jobs(&mut self) -> Vec<(JobId, CronJob)> {
        let now = Utc::now();
        let mut executed = Vec::new();
        let mut to_remove = Vec::new();

        for (id, job) in &mut self.jobs {
            let should_run = match &job.schedule {
                ScheduleType::OneShot(time) => *time <= now && job.run_count == 0,
                ScheduleType::Interval(ms) => job
                    .last_run
                    .map(|last| now.timestamp() - last as i64 >= (*ms as i64 / 1000))
                    .unwrap_or(true),
                ScheduleType::Cron(_expr, _tz) => {
                    // Simplified - real impl would parse cron schedule
                    false
                }
            };

            if should_run {
                job.run_count += 1;
                job.last_run = Some(now.timestamp() as u64);
                executed.push((id.clone(), job.clone()));

                // Remove one-shot jobs after execution
                if matches!(job.schedule, ScheduleType::OneShot(_)) {
                    to_remove.push(id.clone());
                }

                // Remove if max runs reached
                if job
                    .max_runs
                    .map(|max| job.run_count >= max)
                    .unwrap_or(false)
                {
                    to_remove.push(id.clone());
                }
            }
        }

        for id in to_remove {
            self.jobs.remove(&id);
        }

        executed
    }

    pub async fn persist(&self) -> Result<(), CronError> {
        if let Some(persistence) = &self.persistence {
            persistence.save(&self.jobs).await?;
        }
        Ok(())
    }

    pub async fn restore(&mut self) -> Result<(), CronError> {
        if let Some(persistence) = &self.persistence {
            self.jobs = persistence.load().await?;
        }
        Ok(())
    }
}

impl Default for CronScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Cron errors
#[derive(Debug, Clone)]
pub enum CronError {
    PersistenceError(String),
    InvalidSchedule(String),
}

impl std::fmt::Display for CronError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CronError::PersistenceError(s) => write!(f, "Persistence error: {}", s),
            CronError::InvalidSchedule(s) => write!(f, "Invalid schedule: {}", s),
        }
    }
}

impl std::error::Error for CronError {}
