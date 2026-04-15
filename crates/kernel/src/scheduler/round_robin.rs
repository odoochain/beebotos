//! Round-robin scheduler

use super::{ProcessControlBlock, ProcessId, ProcessState, Scheduler};
use std::collections::VecDeque;
use std::time::Duration;

/// Round-robin scheduler
pub struct RoundRobinScheduler {
    ready_queue: VecDeque<ProcessId>,
    time_quantum: Duration,
    current_process: Option<ProcessId>,
    time_slice_remaining: Duration,
}

impl RoundRobinScheduler {
    pub fn new(time_quantum: Duration) -> Self {
        Self {
            ready_queue: VecDeque::new(),
            time_quantum,
            current_process: None,
            time_slice_remaining: time_quantum,
        }
    }

    pub fn with_quantum(mut self, quantum: Duration) -> Self {
        self.time_quantum = quantum;
        self.time_slice_remaining = quantum;
        self
    }

    pub fn get_time_quantum(&self) -> Duration {
        self.time_quantum
    }

    pub fn set_time_quantum(&mut self, quantum: Duration) {
        self.time_quantum = quantum;
    }
}

impl Scheduler for RoundRobinScheduler {
    fn schedule(&mut self, processes: &mut [ProcessControlBlock]) -> Option<ProcessId> {
        if let Some(current_pid) = self.current_process {
            if self.time_slice_remaining > Duration::ZERO {
                if let Some(pcb) = processes.iter().find(|p| p.pid == current_pid) {
                    if pcb.state == ProcessState::Running {
                        return Some(current_pid);
                    }
                }
            }

            if let Some(pcb) = processes.iter_mut().find(|p| p.pid == current_pid) {
                if pcb.state == ProcessState::Running {
                    pcb.state = ProcessState::Ready;
                    self.ready_queue.push_back(current_pid);
                }
            }
        }

        while let Some(pid) = self.ready_queue.pop_front() {
            if let Some(pcb) = processes.iter_mut().find(|p| p.pid == pid) {
                if pcb.state == ProcessState::Ready {
                    pcb.state = ProcessState::Running;
                    self.current_process = Some(pid);
                    self.time_slice_remaining = self.time_quantum;
                    return Some(pid);
                }
            }
        }

        self.current_process = None;
        None
    }

    fn add_process(&mut self, pcb: ProcessControlBlock) {
        self.ready_queue.push_back(pcb.pid);
    }

    fn remove_process(&mut self, pid: ProcessId) {
        self.ready_queue.retain(|&p| p != pid);
        if self.current_process == Some(pid) {
            self.current_process = None;
            self.time_slice_remaining = Duration::ZERO;
        }
    }

    fn tick(&mut self) {
        if self.time_slice_remaining > Duration::ZERO {
            self.time_slice_remaining = self.time_slice_remaining.saturating_sub(Duration::from_millis(1));
        }
    }
}

/// Weighted round-robin scheduler
pub struct WeightedRoundRobinScheduler {
    queues: Vec<VecDeque<ProcessId>>,
    weights: Vec<usize>,
    current_queue: usize,
    current_weight_count: usize,
    time_quantum: Duration,
}

impl WeightedRoundRobinScheduler {
    /// Create new weighted round-robin scheduler
    pub fn new(weights: Vec<usize>, time_quantum: Duration) -> Self {
        let num_queues = weights.len();
        Self {
            queues: (0..num_queues).map(|_| VecDeque::new()).collect(),
            weights,
            current_queue: 0,
            current_weight_count: 0,
            time_quantum,
        }
    }

    pub fn enqueue(&mut self, pid: ProcessId, queue_level: usize) {
        if queue_level < self.queues.len() {
            self.queues[queue_level].push_back(pid);
        }
    }
}

impl Scheduler for WeightedRoundRobinScheduler {
    fn schedule(&mut self, processes: &mut [ProcessControlBlock]) -> Option<ProcessId> {
        let num_queues = self.queues.len();
        
        for _ in 0..num_queues {
            let queue_idx = self.current_queue;
            
            if let Some(pid) = self.queues[queue_idx].pop_front() {
                if let Some(pcb) = processes.iter().find(|p| p.pid == pid) {
                    if pcb.state == ProcessState::Ready {
                        self.current_weight_count += 1;
                        if self.current_weight_count >= self.weights[queue_idx] {
                            self.current_weight_count = 0;
                            self.current_queue = (self.current_queue + 1) % num_queues;
                        }
                        return Some(pid);
                    }
                }
            }
            
            self.current_queue = (self.current_queue + 1) % num_queues;
            self.current_weight_count = 0;
        }
        
        None
    }

    fn add_process(&mut self, pcb: ProcessControlBlock) {
        let queue_idx = match pcb.priority {
            super::Priority::RealTime => 0,
            super::Priority::High => 0,
            super::Priority::Normal => 1,
            super::Priority::Low => 2,
            super::Priority::Idle => 2,
        };
        self.enqueue(pcb.pid, queue_idx.min(self.queues.len() - 1));
    }

    fn remove_process(&mut self, pid: ProcessId) {
        for queue in &mut self.queues {
            queue.retain(|&p| p != pid);
        }
    }

    fn tick(&mut self) {}
}
