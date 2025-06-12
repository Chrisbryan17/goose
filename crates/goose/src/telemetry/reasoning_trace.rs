use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DecisionType {
    // Agent Lifecycle & Planning
    SessionStart,
    PlanGeneration,         // Overall plan creation
    PlanStepSelection,      // Deciding which step of a plan to execute next
    PlanStepExecution,      // The execution of a single plan step (often a tool call)
    GoalAchieved,
    GoalFailed,

    // LLM Interaction
    PromptFinalization,     // Final prompt assembly before sending to LLM
    LlmRequestSent,         // Logging the actual request to LLM
    LlmResponseProcessing,  // Processing raw LLM response
    JustificationGeneration,// LLM was asked to justify a choice

    // Tool Interaction
    ToolSelection,          // LLM or Agent selected a tool
    ToolInputPreparation,   // Parameters for a tool call finalized
    ToolCallDispatch,       // Actual dispatch of a tool call
    ToolResponseProcessing, // Processing raw tool response

    // Capability & Configuration
    ExtensionManagement,    // Adding/removing an extension
    ConfigurationChange,    // Significant config change affecting agent behavior

    // Learning & Adaptation
    FeedbackIngestion,      // Feedback was received
    KnowledgeGapIdentified,
    PromptVariantSelection,

    // Other
    AgentStateChange,       // General state changes not covered above
    ErrorConditionObserved,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReasoningTrace {
    pub trace_id: String, // UUID
    pub session_id: String,
    pub parent_trace_id: Option<String>, // Link to a higher-level trace this is part of
    pub timestamp: DateTime<Utc>,
    pub decision_type: DecisionType,
    pub inputs: Value, // JSON: e.g., sub-goal, available_tools, context, current_prompt_template
    pub alternatives_considered: Option<Value>, // JSON: e.g., list of tool names with scores, plan outlines
    pub selected_alternative: Value, // JSON: e.g., chosen tool name + params, selected plan step details, final prompt sent to LLM
    pub justification_llm_response: Option<String>, // If LLM was asked "Why X?"
    pub confidence_score_llm_self_assessed: Option<f32>, // LLM's own confidence
    pub confidence_score_derived: Option<f32>, // Agent-derived confidence (e.g., from tool history)
    pub outcome: Option<Value>, // JSON: Result of the decision/action (e.g., tool_output_summary, next_state_description, error_message)
    pub duration_ms: Option<u64>, // How long this decision/action took
}

impl ReasoningTrace {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: String,
        parent_trace_id: Option<String>,
        decision_type: DecisionType,
        inputs: Value,
        selected_alternative: Value,
    ) -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            session_id,
            parent_trace_id,
            timestamp: Utc::now(),
            decision_type,
            inputs,
            alternatives_considered: None,
            selected_alternative,
            justification_llm_response: None,
            confidence_score_llm_self_assessed: None,
            confidence_score_derived: None,
            outcome: None,
            duration_ms: None,
        }
    }

    // Builder methods for optional fields
    pub fn with_alternatives(mut self, alternatives: Value) -> Self {
        self.alternatives_considered = Some(alternatives);
        self
    }

    pub fn with_justification(mut self, justification: String) -> Self {
        self.justification_llm_response = Some(justification);
        self
    }

    pub fn with_llm_confidence(mut self, score: f32) -> Self {
        self.confidence_score_llm_self_assessed = Some(score);
        self
    }

    pub fn with_derived_confidence(mut self, score: f32) -> Self {
        self.confidence_score_derived = Some(score);
        self
    }

    pub fn with_outcome(mut self, outcome: Value) -> Self {
        self.outcome = Some(outcome);
        self
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

// Trait for emitting traces
#[async_trait::async_trait]
pub trait TraceEmitter: Send + Sync {
    async fn emit_trace(&self, trace: ReasoningTrace) -> Result<(), String>;
}

// Example: In-memory emitter for testing or simple cases
use std::sync::Mutex as StdMutex;

pub struct InMemoryTraceEmitter {
    traces: Arc<StdMutex<Vec<ReasoningTrace>>>,
}

impl InMemoryTraceEmitter {
    pub fn new() -> Self {
        Self {
            traces: Arc::new(StdMutex::new(Vec::new())),
        }
    }
    pub fn get_traces(&self) -> Vec<ReasoningTrace> {
        self.traces.lock().unwrap().clone()
    }
}

impl Default for InMemoryTraceEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TraceEmitter for InMemoryTraceEmitter {
    async fn emit_trace(&self, trace: ReasoningTrace) -> Result<(), String> {
        self.traces.lock().unwrap().push(trace);
        Ok(())
    }
}

// TODO: Implement AsyncLogTraceEmitter (e.g., writing to structured log files or a remote service)
// pub struct AsyncLogTraceEmitter { /* ... client or file handle ... */ }
// #[async_trait::async_trait]
// impl TraceEmitter for AsyncLogTraceEmitter { /* ... */ }

// Add telemetry mod.rs
pub mod mod_rs {
    // This is a placeholder for a potential mod.rs if telemetry becomes a larger module
    pub fn placeholder() {}
}
