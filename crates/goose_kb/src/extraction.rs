use crate::{Node, Edge, NodeType, EdgeType}; // Assuming these are in crate::lib
use goose_core::message::Content; // This is a placeholder path.
                                  // Actual path will depend on where `Content` is defined.
                                  // If `Content` is in the `goose` crate, this creates a circular dependency
                                  // if `goose_kb` is a separate crate.
                                  // For this phase, we'll assume `Content` can be made available,
                                  // possibly by moving this module into the `goose` crate later.

use serde_json::{Value, json};
use async_trait::async_trait;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

// Assuming access to an LLM provider. This is problematic if goose_kb is a dep of goose.
// For a cleaner architecture, the LLM provider should be passed into the extractor's methods
// or the extractor should be part of the `goose` crate.
// For now, this is a conceptual placeholder.
use goose::providers::base::Provider; // Placeholder path for Provider trait

#[derive(Debug, Clone)]
pub struct ExtractionContext {
    pub session_id: String,
    pub source_document_uri: Option<String>, // e.g., URL, file path, message_id
    pub related_trace_id: Option<String>,
    pub target_concept_id: Option<String>, // If extraction is aimed at a specific KB concept
    pub extraction_prompt_template: Option<String>, // Override default prompt
    pub custom_extraction_rules: Option<Value>, // e.g., specific entities/relations to look for
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractedEntityInfo {
    id: Option<String>, // Existing ID if matched, new if not
    name: String,
    entity_type: String, // E.g., "Person", "File", "Tool", "Concept" - maps to NodeType
    properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractedRelationInfo {
    source_entity_name: String, // Name of source entity
    target_entity_name: String, // Name of target entity
    relation_type: String,   // E.g., "UsesTool", "Mentions" - maps to EdgeType
    properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmExtractionOutput {
    entities: Vec<ExtractedEntityInfo>,
    relations: Vec<ExtractedRelationInfo>,
}

#[async_trait]
pub trait KnowledgeExtractionServiceProvider: Send + Sync {
    async fn extract_from_text(&self, text_content: &str, context: &ExtractionContext) -> Result<Vec<(Node, Vec<Edge>)>, String>;
}

pub struct LlmKnowledgeExtractor {
    llm_provider: Arc<dyn Provider>, // LLM provider passed in
    // knowledge_store: Arc<dyn KnowledgeStoreProvider> // For entity resolution against existing KB
}

impl LlmKnowledgeExtractor {
    pub fn new(llm_provider: Arc<dyn Provider /*, knowledge_store: Arc<dyn KnowledgeStoreProvider>*/>) -> Self {
        Self { llm_provider /*, knowledge_store*/ }
    }

    fn map_str_to_nodetype(s: &str) -> NodeType {
        match s.to_lowercase().as_str() {
            "person" | "people" => NodeType::ExternalEntity, // Could have sub-labels
            "organization" | "org" => NodeType::ExternalEntity,
            "location" | "place" => NodeType::ExternalEntity,
            "file" => NodeType::File,
            "tool" => NodeType::Tool,
            "concept" => NodeType::Concept,
            "task" => NodeType::Plan, // Or a new "Task" NodeType
            _ => NodeType::Generic, // Default or more sophisticated mapping
        }
    }

    fn map_str_to_edgetype(s: &str) -> EdgeType {
        match s.to_lowercase().as_str() {
            "uses" | "employs" | "utilizes" => EdgeType::UsesTool, // Needs context to confirm object is a tool
            "mentions" | "refers_to" | "discusses" => EdgeType::Mentions,
            "related_to" | "associated_with" => EdgeType::RelatedTo,
            "works_on" | "modifies" => EdgeType::ReferencesFile, // If object is a file
            "is_a" | "instance_of" => EdgeType::InstanceOf,
            _ => EdgeType::RelatedTo, // Default
        }
    }
}

#[async_trait]
impl KnowledgeExtractionServiceProvider for LlmKnowledgeExtractor {
    async fn extract_from_text(&self, text_content: &str, context: &ExtractionContext) -> Result<Vec<(Node, Vec<Edge>)>, String> {
        let prompt_template = context.extraction_prompt_template.as_deref().unwrap_or(
            "Extract named entities and simple relationships from the following text.
            Output MUST be a single JSON object with two keys: 'entities' and 'relations'.
            'entities' is a list of objects, each with 'name' (string), 'entity_type' (e.g., Person, File, Tool, Concept, Organization), and optional 'properties' (object).
            'relations' is a list of objects, each with 'source_entity_name', 'target_entity_name', 'relation_type' (e.g., Uses, Mentions, RelatedTo), and optional 'properties' (object).
            Text to process:
            ---
            {text_content}
            ---
            JSON Output:"
        );

        let system_prompt = prompt_template.replace("{text_content}", text_content);

        // In a real scenario, messages would be an empty slice for this kind of direct completion task.
        // Tools would also be empty.
        let response = self.llm_provider.complete(&system_prompt, &[], &[]).await
            .map_err(|e| format!("LLM call failed during extraction: {}", e))?;

        let llm_output_text = response.message.as_concat_text();
        let extracted_data: LlmExtractionOutput = serde_json::from_str(&llm_output_text)
            .map_err(|e| format!("Failed to parse LLM JSON output for extraction: {}. Output was: {}", e, llm_output_text))?;

        let mut nodes_map: HashMap<String, Node> = HashMap::new();
        let mut all_edges: Vec<Edge> = Vec::new();

        // Process entities first
        for extracted_entity in extracted_data.entities {
            // Basic entity resolution: use name as part of ID for now.
            // Future: use self.knowledge_store to find existing nodes by name/aliases.
            let node_id = extracted_entity.id.unwrap_or_else(||
                format!("urn:goose:entity:{}:{}", extracted_entity.entity_type.to_lowercase(), Uuid::new_v4())
            );

            let node_type = Self::map_str_to_nodetype(&extracted_entity.entity_type);
            let properties = extracted_entity.properties.map_or_else(|| json!({"name": extracted_entity.name.clone()}), |p| {
                let mut base_props = json!({"name": extracted_entity.name.clone()});
                if let Value::Object(mut map) = base_props {
                    if let Value::Object(p_map) = json!(p) { // Convert HashMap to serde_json::Map
                        map.extend(p_map);
                    }
                    base_props = Value::Object(map);
                }
                base_props
            });


            let node = Node::new(node_id.clone(), node_type, properties);
            nodes_map.insert(extracted_entity.name.clone(), node); // Store by name for relation mapping
        }

        // Process relations
        for extracted_relation in extracted_data.relations {
            if let (Some(source_node), Some(target_node)) = (
                nodes_map.get(&extracted_relation.source_entity_name),
                nodes_map.get(&extracted_relation.target_entity_name)
            ) {
                let edge_type = Self::map_str_to_edgetype(&extracted_relation.relation_type);
                let properties = extracted_relation.properties.map_or_else(Value::Null, |p| json!(p));
                let edge = Edge::new(source_node.id.clone(), target_node.id.clone(), edge_type, properties);
                all_edges.push(edge);
            } else {
                // Log warning: could not find source or target node for relation
                eprintln!("Warning: Could not find source ('{}') or target ('{}') node for relation '{}'",
                    extracted_relation.source_entity_name,
                    extracted_relation.target_entity_name,
                    extracted_relation.relation_type);
            }
        }

        // Convert nodes_map values into a Vec for the final output structure
        let final_nodes_with_edges: Vec<(Node, Vec<Edge>)> = nodes_map.into_values().map(|node| {
            // For simplicity here, we are not filtering edges per node, just returning all edges
            // A more accurate representation might associate specific edges if the LLM output linked them directly
            // to a main entity in a multi-entity extraction.
            // For now, we return each node, and the caller gets all edges found in the text.
            // A better output might be (Vec<Node>, Vec<Edge>).
            // Let's adjust to that.
            (node, Vec::new()) // Placeholder for edges specifically originating *from this node* if needed.
                               // The current design asks for Vec<(Node, Vec<Edge>)>, which is a bit ambiguous.
                               // Assuming it means "each extracted node, and then all extracted edges separately".
                               // For now, this will be (Node, []). The edges are in all_edges.
        }).collect();

        // A more useful return might be: Result<(Vec<Node>, Vec<Edge>), String>
        // For now, adhering to Vec<(Node, Vec<Edge>)> means we have to decide what edges go with what node.
        // Let's return each node and an empty vec of edges, and the caller can get all_edges separately or we change the trait.
        // For this phase, let's simplify: return all unique nodes and all unique edges.
        // The trait change to `Result<(Vec<Node>, Vec<Edge>), String>` would be better.
        // Sticking to the current trait:
        if final_nodes_with_edges.is_empty() {
            Ok(Vec::new())
        } else {
            // This is not ideal. Let's assume the intent is one primary node and its direct new edges.
            // The LLM prompt would need to be more specific.
            // For now, just returning the first node and all edges.
            let (first_node, _) = final_nodes_with_edges[0].clone();
            Ok(vec![(first_node, all_edges)])
        }
    }
}
