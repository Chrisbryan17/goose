use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum PlanStatus {
    Pending,
    Ready, // All preconditions met, ready for execution or further breakdown
    InProgress,
    CompletedSuccessfully,
    Failed,
    CancelledByUser,
    RequiresHumanIntervention,
    WaitingForDependency, // Waiting for another step/plan to complete
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategicGoal {
    pub id: String,
    pub user_request_summary: String, // Brief summary of the user request
    pub description: String,          // LLM-generated strategic goal
    pub status: PlanStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tactical_plan_ids: Vec<String>,
    pub properties: Option<Value>, // For acceptance criteria, overall constraints
    pub original_user_message_id: Option<String>, // Link to user's message
}

impl StrategicGoal {
    pub fn new(user_request_summary: String, description: String, original_user_message_id: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_request_summary,
            description,
            status: PlanStatus::Pending,
            created_at: now,
            updated_at: now,
            tactical_plan_ids: Vec::new(),
            properties: None,
            original_user_message_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TacticalPlan {
    pub id: String,
    pub strategic_goal_id: String,
    pub description: String,
    pub status: PlanStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub operational_step_ids: Vec<String>,
    pub preconditions: Vec<String>, // For Phase 2, simplified as string descriptions
    pub effects: Vec<String>,       // For Phase 2, simplified as string descriptions
    pub priority: Option<u8>,       // Optional priority for ordering tactical plans
    pub properties: Option<Value>,
}

impl TacticalPlan {
    pub fn new(strategic_goal_id: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            strategic_goal_id,
            description,
            status: PlanStatus::Pending,
            created_at: now,
            updated_at: now,
            operational_step_ids: Vec::new(),
            preconditions: Vec::new(),
            effects: Vec::new(),
            priority: None,
            properties: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperationalStep {
    pub id: String,
    pub tactical_plan_id: String,
    pub description: String,
    pub tool_name: Option<String>,
    pub tool_parameters: Option<Value>,
    pub human_action_description: Option<String>, // If step requires human action
    pub status: PlanStatus,
    pub expected_outcome_description: String, // LLM's prediction of what should happen
    pub actual_outcome_description: Option<String>, // Result after execution
    pub execution_attempts: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub depends_on_step_ids: Vec<String>, // IDs of other OperationalSteps that must complete first
    pub output_parameters: Option<Value>, // Key outputs from this step to be used by subsequent steps
    pub properties: Option<Value>, // For retry policies, error handling notes
}

impl OperationalStep {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tactical_plan_id: String,
        description: String,
        expected_outcome_description: String,
        tool_name: Option<String>,
        tool_parameters: Option<Value>,
        human_action_description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            tactical_plan_id,
            description,
            tool_name,
            tool_parameters,
            human_action_description,
            status: PlanStatus::Pending,
            expected_outcome_description,
            actual_outcome_description: None,
            execution_attempts: 0,
            created_at: now,
            updated_at: now,
            depends_on_step_ids: Vec::new(),
            output_parameters: None,
            properties: None,
        }
    }
}

// Module file
pub mod mod_rs {
    pub fn placeholder() {}
}
