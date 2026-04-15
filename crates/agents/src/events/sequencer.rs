//! Event Sequencer

use crate::error::Result;
use std::collections::VecDeque;

/// Sequenced event
#[derive(Debug)]
pub struct SequencedEvent {
    pub sequence: u64,
    pub event: super::bus::Event,
}

/// Event sequencer
pub struct EventSequencer {
    sequence: u64,
    buffer: VecDeque<SequencedEvent>,
}

impl EventSequencer {
    pub fn new() -> Self {
        Self {
            sequence: 0,
            buffer: VecDeque::new(),
        }
    }

    pub fn sequence(&mut self, event: super::bus::Event) -> SequencedEvent {
        self.sequence += 1;
        let sequenced = SequencedEvent {
            sequence: self.sequence,
            event,
        };
        self.buffer.push_back(sequenced.clone());
        sequenced
    }

    pub fn next(&mut self) -> Option<SequencedEvent> {
        self.buffer.pop_front()
    }

    pub fn current_sequence(&self) -> u64 {
        self.sequence
    }
}

impl Default for EventSequencer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SequencedEvent {
    fn clone(&self) -> Self {
        unimplemented!("Event clone not implemented")
    }
}
