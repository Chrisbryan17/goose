use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FeedbackSource {
    ExplicitUI,        // User clicked a button, filled a form
    UserCommand,       // User typed a /feedback command
    ImplicitSentiment, // Derived from user text analysis (future)
    ToolInternalError, // Tool reported a structured error
    AgentObservation,  // Agent inferred feedback (e.g., task completion, repeated errors)
    SystemEvent,       // e.g., unhandled error, critical performance issue
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeedbackEntry {
    pub feedback_id: String, // UUID
    pub session_id: String,
    pub user_id: Option<String>, // If available
    pub related_trace_id: Option<String>, // Link to a ReasoningTrace
    pub related_log_id: Option<String>, // Link to a more general interaction log if different
    pub timestamp: DateTime<Utc>,
    pub source: FeedbackSource,
    pub user_rating_stars: Option<u8>, // 1-5
    pub correction_suggestion_text: Option<String>, // User's suggested improvement
    pub is_error_report: bool,
    pub custom_tags: Vec<String>, // e.g., "planning_issue", "tool_A_failed"
    pub feedback_data: Value, // For arbitrary structured feedback (e.g., tool error details, sentiment scores)
}

impl FeedbackEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: String,
        user_id: Option<String>,
        source: FeedbackSource,
        feedback_data: Value,
    ) -> Self {
        Self {
            feedback_id: Uuid::new_v4().to_string(),
            session_id,
            user_id,
            related_trace_id: None,
            related_log_id: None,
            timestamp: Utc::now(),
            source,
            user_rating_stars: None,
            correction_suggestion_text: None,
            is_error_report: false,
            custom_tags: Vec::new(),
            feedback_data,
        }
    }

    // Builder methods
    pub fn with_related_trace_id(mut self, trace_id: String) -> Self {
        self.related_trace_id = Some(trace_id);
        self
    }
    // ... other builder methods for optional fields ...
    pub fn with_rating_stars(mut self, stars: u8) -> Self {
        self.user_rating_stars = Some(stars.clamp(1, 5));
        self
    }
    pub fn with_correction(mut self, correction: String) -> Self {
        self.correction_suggestion_text = Some(correction);
        self
    }
    pub fn as_error_report(mut self, is_error: bool) -> Self {
        self.is_error_report = is_error;
        self
    }
    pub fn add_tag(mut self, tag: String) -> Self {
        self.custom_tags.push(tag);
        self
    }
}

#[async_trait::async_trait]
pub trait FeedbackStoreProvider: Send + Sync {
    async fn store_feedback(&self, entry: FeedbackEntry) -> Result<(), String>;
    async fn get_feedback_by_id(&self, feedback_id: &str) -> Result<Option<FeedbackEntry>, String>;
    async fn get_feedback_for_session(&self, session_id: &str, limit: Option<usize>) -> Result<Vec<FeedbackEntry>, String>;
    async fn get_feedback_by_trace_id(&self, trace_id: &str) -> Result<Vec<FeedbackEntry>, String>;
    // Example query:
    // async fn query_feedback(&self, query_params: HashMap<String, Value>, limit: Option<usize>) -> Result<Vec<FeedbackEntry>, String>;
}

// Example: In-memory feedback store for testing
use std::sync::Mutex as StdMutex;

pub struct InMemoryFeedbackStore {
    feedback_entries: Arc<StdMutex<HashMap<String, FeedbackEntry>>>,
}

impl InMemoryFeedbackStore {
    pub fn new() -> Self {
        Self {
            feedback_entries: Arc::new(StdMutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryFeedbackStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl FeedbackStoreProvider for InMemoryFeedbackStore {
    async fn store_feedback(&self, entry: FeedbackEntry) -> Result<(), String> {
        self.feedback_entries
            .lock()
            .unwrap()
            .insert(entry.feedback_id.clone(), entry);
        Ok(())
    }

    async fn get_feedback_by_id(&self, feedback_id: &str) -> Result<Option<FeedbackEntry>, String> {
        Ok(self.feedback_entries.lock().unwrap().get(feedback_id).cloned())
    }

    async fn get_feedback_for_session(&self, session_id: &str, limit: Option<usize>) -> Result<Vec<FeedbackEntry>, String> {
        let entries = self.feedback_entries.lock().unwrap();
        let mut results: Vec<FeedbackEntry> = entries
            .values()
            .filter(|e| e.session_id == session_id)
            .cloned()
            .collect();
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Sort descending by time
        if let Some(l) = limit {
            results.truncate(l);
        }
        Ok(results)
    }

    async fn get_feedback_by_trace_id(&self, trace_id: &str) -> Result<Vec<FeedbackEntry>, String> {
        let entries = self.feedback_entries.lock().unwrap();
        let results: Vec<FeedbackEntry> = entries
            .values()
            .filter(|e| e.related_trace_id.as_deref() == Some(trace_id))
            .cloned()
            .collect();
        Ok(results)
    }
}

// Add feedback mod.rs
pub mod mod_rs {
    pub fn placeholder() {}
}
