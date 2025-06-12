use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Arc;

// Using existing PromptManager's location for now, can be moved to a dedicated
// module later if it grows significantly. This keeps it close to PromptManager.
// Alternatively, this could be in `crates/goose/src/learning/prompt_optimization.rs`

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromptVariant {
    pub variant_id: String, // UUID or hash of content
    pub prompt_type_key: String, // e.g., "SystemPrompt_Main", "PlanningPrompt_DeveloperExtension", "ToolSelectionClarification"
    pub template_text: String,
    pub description: Option<String>, // Why this variant exists, what it's trying to achieve
    pub version: u32, // Increment on significant changes to the same conceptual variant
    pub author_id: Option<String>, // Who created/last modified this variant
    pub creation_date: DateTime<Utc>,
    pub last_used_date: Option<DateTime<Utc>>,
    // Metrics could be simple key-value or more structured if needed
    pub performance_metrics: HashMap<String, f64>, // e.g., "success_rate", "avg_tokens_total", "avg_user_rating", "conversion_to_goal_rate", "execution_count"
    pub experiment_group: Option<String>, // For A/B testing, e.g., "control", "treatment_A"
    pub is_active: bool, // True if this is a candidate for selection (either default or for experiments)
    pub deprecation_date: Option<DateTime<Utc>>, // If this variant is no longer recommended
}

impl PromptVariant {
    pub fn new(prompt_type_key: String, template_text: String, author_id: Option<String>) -> Self {
        Self {
            variant_id: Uuid::new_v4().to_string(),
            prompt_type_key,
            template_text,
            description: None,
            version: 1,
            author_id,
            creation_date: Utc::now(),
            last_used_date: None,
            performance_metrics: HashMap::new(),
            experiment_group: None,
            is_active: true,
            deprecation_date: None,
        }
    }
}

#[async_trait::async_trait]
pub trait PromptVariantProvider: Send + Sync {
    // In Phase 1, get_active_variant might just fetch the one with highest version or a specific flag.
    // True learning/selection algorithm comes later.
    async fn get_active_variant(
        &self,
        prompt_type_key: &str,
        // context: Option<Value> // Future: context for more sophisticated selection
    ) -> Result<Option<PromptVariant>, String>;

    async fn get_variant_by_id(&self, variant_id: &str) -> Result<Option<PromptVariant>, String>;
    async fn store_variant(&self, variant: &PromptVariant) -> Result<(), String>; // Upsert logic
    async fn update_variant_metrics(&self, variant_id: &str, metrics_update: HashMap<String, f64>, increment_execution_count: bool) -> Result<(), String>;
    async fn list_variants_for_type(&self, prompt_type_key: &str, include_inactive: bool) -> Result<Vec<PromptVariant>, String>;
    // More advanced methods for A/B testing setup might be added later.
}


// Example: In-memory store for testing
use std::sync::Mutex as StdMutex;

pub struct InMemoryPromptVariantProvider {
    variants: Arc<StdMutex<HashMap<String, PromptVariant>>>,
}

impl InMemoryPromptVariantProvider {
    pub fn new() -> Self {
        Self {
            variants: Arc::new(StdMutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryPromptVariantProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl PromptVariantProvider for InMemoryPromptVariantProvider {
    async fn get_active_variant(
        &self,
        prompt_type_key: &str,
    ) -> Result<Option<PromptVariant>, String> {
        let variants_map = self.variants.lock().unwrap();
        // Phase 1: Simple logic - get the highest version, active variant for the type_key
        // More sophisticated selection (A/B testing, bandit algorithm) in later phases.
        let active_variant = variants_map
            .values()
            .filter(|v| v.prompt_type_key == prompt_type_key && v.is_active && v.deprecation_date.is_none())
            .max_by_key(|v| v.version); // Could also sort by last_updated or a specific "priority" field
        Ok(active_variant.cloned())
    }

    async fn get_variant_by_id(&self, variant_id: &str) -> Result<Option<PromptVariant>, String> {
        Ok(self.variants.lock().unwrap().get(variant_id).cloned())
    }

    async fn store_variant(&self, variant: &PromptVariant) -> Result<(), String> {
        self.variants
            .lock()
            .unwrap()
            .insert(variant.variant_id.clone(), variant.clone());
        Ok(())
    }

    async fn update_variant_metrics(
        &self,
        variant_id: &str,
        metrics_update: HashMap<String, f64>,
        increment_execution_count: bool
    ) -> Result<(), String> {
        let mut variants_map = self.variants.lock().unwrap();
        if let Some(variant) = variants_map.get_mut(variant_id) {
            for (key, value) in metrics_update {
                // This could be simple replacement or more complex like averaging, summing, etc.
                // For now, let's assume direct update for simplicity in Phase 1.
                variant.performance_metrics.insert(key, value);
            }
            if increment_execution_count {
                let count = variant.performance_metrics.entry("execution_count".to_string()).or_insert(0.0);
                *count += 1.0;
            }
            variant.last_used_date = Some(Utc::now());
            Ok(())
        } else {
            Err(format!("Variant with id {} not found", variant_id))
        }
    }

    async fn list_variants_for_type(&self, prompt_type_key: &str, include_inactive: bool) -> Result<Vec<PromptVariant>, String> {
        let variants_map = self.variants.lock().unwrap();
        let results = variants_map
            .values()
            .filter(|v| v.prompt_type_key == prompt_type_key && (include_inactive || (v.is_active && v.deprecation_date.is_none())))
            .cloned()
            .collect();
        Ok(results)
    }
}

// Add prompt_variants mod.rs
pub mod mod_rs {
    pub fn placeholder() {}
}
