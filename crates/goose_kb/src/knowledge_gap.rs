use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum KnowledgeGapStatus {
    Open,                       // Newly identified
    InvestigatingWithTools,     // Agent is actively trying to resolve it using tools
    WaitingForUserInput,        // Agent has asked the user for clarification/information
    ResolvedByAgent,            // Agent believes it has filled the gap
    ResolvedByExternalData,     // User provided the necessary information, or it was found externally
    CannotResolveCurrently,     // Agent has tried and cannot resolve it with current capabilities
    PendingDeveloperReview,     // Systematically flagged for offline review
    ClosedStale,                // Closed due to inactivity or irrelevance
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeGapEntry {
    pub gap_id: String, // UUID
    pub session_id: String,
    pub timestamp_identified: DateTime<Utc>,
    pub description_by_llm_or_agent: String, // What is missing or misunderstood
    pub type_of_gap: Option<String>, // E.g., "MissingSpecificFact", "ToolCapabilityLacking", "AmbiguousConcept", "UncertainCondition"
    pub status: KnowledgeGapStatus,
    pub related_trace_id: Option<String>, // ReasoningTrace that led to identification
    pub context_snapshot: Option<serde_json::Value>, // JSON snapshot of relevant agent state or plan step
    pub resolution_attempts: u32,
    pub resolution_details: Option<String>, // How it was (or wasn't) resolved
    pub last_investigation_utc: Option<DateTime<Utc>>,
    pub priority: Option<u8>, // 1-5, higher is more critical
}

impl KnowledgeGapEntry {
    pub fn new(
        session_id: String,
        description: String,
        related_trace_id: Option<String>,
        context_snapshot: Option<serde_json::Value>,
        type_of_gap: Option<String>,
    ) -> Self {
        Self {
            gap_id: Uuid::new_v4().to_string(),
            session_id,
            timestamp_identified: Utc::now(),
            description_by_llm_or_agent: description,
            type_of_gap,
            status: KnowledgeGapStatus::Open,
            related_trace_id,
            context_snapshot,
            resolution_attempts: 0,
            resolution_details: None,
            last_investigation_utc: None,
            priority: None,
        }
    }
}
