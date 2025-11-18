use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<WorkflowTrigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<WorkflowInput>,
    pub steps: Vec<WorkflowStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_handling: Option<Vec<ErrorHandler>>,
}

/// Workflow trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WorkflowTrigger {
    Manual,
    Schedule { cron: String },
    Webhook { path: String },
}

/// Workflow input schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInput {
    pub schema: serde_json::Value,
}

/// A single step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel: Option<u32>,
    pub instructions: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_failure: Option<String>,
}

/// Retry configuration for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff: Option<BackoffStrategy>,
}

/// Backoff strategy for retries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackoffStrategy {
    Linear,
    Exponential,
    Fixed,
}

/// Error handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandler {
    pub step: String,
    pub agent: String,
    pub instructions: String,
}

/// A workflow execution instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: String,
    pub workflow_name: String,
    pub status: WorkflowRunStatus,
    pub input: Option<serde_json::Value>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub steps: Vec<StepExecution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Status of a workflow run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for WorkflowRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowRunStatus::Pending => write!(f, "pending"),
            WorkflowRunStatus::Running => write!(f, "running"),
            WorkflowRunStatus::Completed => write!(f, "completed"),
            WorkflowRunStatus::Failed => write!(f, "failed"),
            WorkflowRunStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Execution state of a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {
    pub step_name: String,
    pub status: StepStatus,
    pub agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub attempt: u32,
}

/// Status of a step execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Retrying,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatus::Pending => write!(f, "pending"),
            StepStatus::Running => write!(f, "running"),
            StepStatus::Completed => write!(f, "completed"),
            StepStatus::Failed => write!(f, "failed"),
            StepStatus::Skipped => write!(f, "skipped"),
            StepStatus::Retrying => write!(f, "retrying"),
        }
    }
}

/// Workflow execution context
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub run_id: String,
    pub workflow_name: String,
    pub input: serde_json::Value,
    pub step_outputs: HashMap<String, serde_json::Value>,
}

impl WorkflowContext {
    pub fn new(run_id: String, workflow_name: String, input: serde_json::Value) -> Self {
        Self {
            run_id,
            workflow_name,
            input,
            step_outputs: HashMap::new(),
        }
    }

    pub fn add_step_output(&mut self, step_name: String, output: serde_json::Value) {
        self.step_outputs.insert(step_name, output);
    }

    pub fn get_step_output(&self, step_name: &str) -> Option<&serde_json::Value> {
        self.step_outputs.get(step_name)
    }
}
