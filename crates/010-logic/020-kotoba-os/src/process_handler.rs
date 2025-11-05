//! ProcessHandler implementation for KotobaOS
//!
//! Orchestrates the execution of a process network from a story.

use crate::types::{Process, Story};
use crate::Result;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

/// ProcessHandler orchestrates process network execution
pub struct ProcessHandler {
    /// Story containing the process network
    story: Story,
}

impl ProcessHandler {
    /// Create a new process handler
    pub fn new(story: Story) -> Self {
        Self { story }
    }

    /// Get the story
    pub fn story(&self) -> &Story {
        &self.story
    }

    /// Get the ordered list of processes to execute
    /// 
    /// Finds the initial process (one not referenced by any other process's `next` property)
    /// and returns the chain of processes in execution order.
    pub fn get_process_chain(&self) -> Vec<Process> {
        let processes = self.story.extract_processes();
        
        if processes.is_empty() {
            return Vec::new();
        }

        // Build a map of process IDs to processes
        let process_map: HashMap<String, Process> = processes
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();

        // Find initial processes (ones not referenced by any other process's `next`)
        let referenced: HashSet<String> = process_map
            .values()
            .filter_map(|p| p.next.as_ref())
            .cloned()
            .collect();

        let initial_processes: Vec<Process> = process_map
            .values()
            .filter(|p| !referenced.contains(&p.id))
            .cloned()
            .collect();

        if initial_processes.is_empty() {
            // If no initial process found, return all processes
            process_map.values().cloned().collect()
        } else {
            // Build chains from initial processes
            let mut chains = Vec::new();
            for initial in initial_processes {
                let mut chain = Vec::new();
                let mut current = Some(initial);
                
                while let Some(process) = current {
                    chain.push(process.clone());
                    if let Some(next_id) = &process.next {
                        current = process_map.get(next_id).cloned();
                    } else {
                        break;
                    }
                }
                
                chains.extend(chain);
            }
            
            chains
        }
    }
}

