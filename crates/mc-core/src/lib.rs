//! Core runtime interfaces.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub backend: String,
    pub planner_mode: String,
}

impl ExecutionPlan {
    pub fn new(backend: impl Into<String>, planner_mode: impl Into<String>) -> Self {
        Self {
            backend: backend.into(),
            planner_mode: planner_mode.into(),
        }
    }
}
