use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use async_trait::async_trait; // Ensure this is in Cargo.toml for goose_kb

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    // Core Entities
    Agent,
    User,
    Session,

    // Execution & Provenance
    Tool,           // Definition of a tool
    ToolCall,       // An instance of a tool being called
    ReasoningTrace, // Link to a reasoning trace document/event
    Feedback,       // Link to a feedback entry

    // Data & Content
    File,
    WebResource,
    Directory,
    Message,        // A specific message in a session

    // Abstract & Learned
    Concept,        // Abstract concept learned or defined
    Skill,          // Abstract capability of an agent or tool
    Plan,           // A high-level plan or recipe
    PlanStep,       // A step within a plan

    // External
    ExternalEntity, // A named entity from the real world (e.g., company, person)
    SoftwarePackage,
    ApiEndpoint,

    // Other
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeType {
    // Session & Control Flow
    Initiated,          // (User) -> Initiated -> (Session)
    BelongsToSession,   // (Message | ToolCall | ReasoningTrace | Feedback) -> BelongsToSession -> (Session)
    ExecutedByUser,     // (ToolCall | Plan) -> ExecutedByUser -> (User)
    ExecutedByAgent,    // (ToolCall | Plan) -> ExecutedByAgent -> (Agent)
    PartOfPlan,         // (PlanStep) -> PartOfPlan -> (Plan)
    NextStep,           // (PlanStep) -> NextStep -> (PlanStep)
    Triggers,           // (ReasoningTrace) -> Triggers -> (Decision | Action)

    // Tool & Capability Related
    HasCapability,      // (Agent | Extension) -> HasCapability -> (Tool | Skill)
    UsesTool,           // (PlanStep | ToolCall) -> UsesTool -> (Tool)
    RecommendsTool,     // (Agent | ReasoningTrace) -> RecommendsTool -> (Tool)

    // Data & Information Flow
    Mentions,           // (Message | File) -> Mentions -> (Entity | Concept)
    ReferencesFile,     // (Message | ToolCall) -> ReferencesFile -> (File)
    ReferencesResource, // (Message | ToolCall) -> ReferencesResource -> (WebResource)
    OutputFile,         // (ToolCall) -> OutputFile -> (File)
    InputTo,            // (File | Concept) -> InputTo -> (ToolCall | PlanStep)
    OutputFrom,         // (File | Concept) -> OutputFrom -> (ToolCall | PlanStep)

    // Knowledge & Learning
    HasKnowledgeAbout,  // (Agent | User) -> HasKnowledgeAbout -> (Concept | Entity)
    LearnedFrom,        // (Concept | Skill | Plan) -> LearnedFrom -> (Session | Feedback | ReasoningTrace)
    RelatedTo,          // (Concept | Entity | File) -> RelatedTo -> (Concept | Entity | File) (generic)
    InstanceOf,         // (Entity) -> InstanceOf -> (Concept)
    SubConceptOf,       // (Concept) -> SubConceptOf -> (Concept)

    // Feedback & Evaluation
    ProvidesFeedbackOn, // (Feedback) -> ProvidesFeedbackOn -> (Message | ToolCall | Session)
    AssociatedWith,     // (ReasoningTrace | Feedback) -> AssociatedWith -> (Node) (generic association)

    // File System
    ContainsFile,       // (Directory) -> ContainsFile -> (File)
    ParentDirectory,    // (File | Directory) -> ParentDirectory -> (Directory)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String, // URI or UUID, e.g., "urn:goose:user:123", "urn:goose:file:/path/to/file"
    pub node_type: NodeType,
    pub labels: Vec<String>, // Additional labels for easier querying, e.g., ["Person", "Employee"] for a User node
    pub properties: Value,   // JSON Value for arbitrary properties
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Node {
    pub fn new(id: String, node_type: NodeType, properties: Value) -> Self {
        let now = Utc::now();
        Node {
            id,
            node_type,
            labels: vec![format!("{:?}", node_type)], // Default label from type
            properties,
            created_at: now,
            updated_at: now,
        }
    }
     pub fn new_with_labels(id: String, node_type: NodeType, labels: Vec<String>, properties: Value) -> Self {
        let now = Utc::now();
        Node {
            id,
            node_type,
            labels,
            properties,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String, // UUID
    pub source_node_id: String,
    pub target_node_id: String,
    pub edge_type: EdgeType,
    pub properties: Value, // JSON Value for arbitrary properties like weight, timestamp, source_of_info
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Edge {
    pub fn new(source_node_id: String, target_node_id: String, edge_type: EdgeType, properties: Value) -> Self {
        let now = Utc::now();
        Edge {
            id: Uuid::new_v4().to_string(),
            source_node_id,
            target_node_id,
            edge_type,
            properties,
            created_at: now,
            updated_at: now,
        }
    }
}

#[async_trait]
pub trait KnowledgeStoreProvider: Send + Sync {
    async fn add_node(&self, node: &Node) -> Result<(), String>;
    async fn add_edge(&self, edge: &Edge) -> Result<(), String>;

    async fn get_node_by_id(&self, node_id: &str) -> Result<Option<Node>, String>;
    async fn get_edges_by_node_id(&self, node_id: &str, direction: Option<String>) -> Result<Vec<Edge>, String>; // direction: "incoming", "outgoing", "both"

    // Basic query for nodes by type and property value.
    // More complex queries might need specific methods or a query language interface.
    async fn get_nodes_by_type_and_property(
        &self,
        node_type: NodeType,
        property_key: &str,
        property_value: &Value,
    ) -> Result<Vec<Node>, String>;

    // Example for Cypher if using a Cypher-compatible DB like Neo4j or Memgraph
    async fn query_cypher(&self, query: &str, params: Option<HashMap<String, Value>>) -> Result<Vec<HashMap<String, Value>>, String>;

    // A more generic query method might be useful for other graph DBs
    // async fn query_custom(&self, query_language: &str, query_string: &str, params: Option<Value>) -> Result<Value, String>;

    async fn update_node_properties(&self, node_id: &str, properties_to_update: Value) -> Result<(), String>;
    async fn update_edge_properties(&self, edge_id: &str, properties_to_update: Value) -> Result<(), String>;

    async fn delete_node(&self, node_id: &str) -> Result<(), String>;
    async fn delete_edge(&self, edge_id: &str) -> Result<(), String>;
}

// Placeholder for mod.rs if this becomes its own crate
// pub mod mod_rs {
//     pub fn placeholder() {}
}
pub mod knowledge_gap;
pub mod extraction; // Add this line

// Re-export new structs
pub use knowledge_gap::{KnowledgeGapEntry, KnowledgeGapStatus};
pub use extraction::{ExtractionContext, KnowledgeExtractionServiceProvider, LlmKnowledgeExtractor}; // Add this line

// Example InMemoryKnowledgeStore for testing
use std::sync::Mutex as StdMutex;
use std::collections::HashSet;

pub struct InMemoryKnowledgeStore {
    nodes: Arc<StdMutex<HashMap<String, Node>>>,
    edges: Arc<StdMutex<HashMap<String, Edge>>>,
    // Adjacency lists for faster edge traversal by node
    adj_outgoing: Arc<StdMutex<HashMap<String, HashSet<String>>>>, // node_id -> set of edge_ids
    adj_incoming: Arc<StdMutex<HashMap<String, HashSet<String>>>>, // node_id -> set of edge_ids
}

// ... (InMemoryKnowledgeStore implementation would go here)
// This would be quite involved to correctly implement all graph operations.
// For Phase 1, the focus is the trait and schema. A real graph DB is recommended.

impl InMemoryKnowledgeStore {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(StdMutex::new(HashMap::new())),
            edges: Arc::new(StdMutex::new(HashMap::new())),
            adj_outgoing: Arc::new(StdMutex::new(HashMap::new())),
            adj_incoming: Arc::new(StdMutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryKnowledgeStore {
     fn default() -> Self {
        Self::new()
    }
}


#[async_trait]
impl KnowledgeStoreProvider for InMemoryKnowledgeStore {
    async fn add_node(&self, node: &Node) -> Result<(), String> {
        let mut nodes = self.nodes.lock().unwrap();
        if nodes.contains_key(&node.id) {
            return Err(format!("Node with id {} already exists", node.id));
        }
        nodes.insert(node.id.clone(), node.clone());
        self.adj_outgoing.lock().unwrap().entry(node.id.clone()).or_default();
        self.adj_incoming.lock().unwrap().entry(node.id.clone()).or_default();
        Ok(())
    }

    async fn add_edge(&self, edge: &Edge) -> Result<(), String> {
        let mut edges = self.edges.lock().unwrap();
        if edges.contains_key(&edge.id) {
            return Err(format!("Edge with id {} already exists", edge.id));
        }
        // Ensure source and target nodes exist
        let nodes = self.nodes.lock().unwrap();
        if !nodes.contains_key(&edge.source_node_id) {
            return Err(format!("Source node {} not found for edge {}", edge.source_node_id, edge.id));
        }
        if !nodes.contains_key(&edge.target_node_id) {
            return Err(format!("Target node {} not found for edge {}", edge.target_node_id, edge.id));
        }
        drop(nodes); // Release lock

        edges.insert(edge.id.clone(), edge.clone());
        self.adj_outgoing.lock().unwrap().entry(edge.source_node_id.clone()).or_default().insert(edge.id.clone());
        self.adj_incoming.lock().unwrap().entry(edge.target_node_id.clone()).or_default().insert(edge.id.clone());
        Ok(())
    }

    async fn get_node_by_id(&self, node_id: &str) -> Result<Option<Node>, String> {
        Ok(self.nodes.lock().unwrap().get(node_id).cloned())
    }

    async fn get_edges_by_node_id(&self, node_id: &str, direction: Option<String>) -> Result<Vec<Edge>, String> {
        let mut result_edge_ids = HashSet::new();
        let dir_str = direction.as_deref().unwrap_or("both");

        if dir_str == "outgoing" || dir_str == "both" {
            if let Some(ids) = self.adj_outgoing.lock().unwrap().get(node_id) {
                result_edge_ids.extend(ids.clone());
            }
        }
        if dir_str == "incoming" || dir_str == "both" {
             if let Some(ids) = self.adj_incoming.lock().unwrap().get(node_id) {
                result_edge_ids.extend(ids.clone());
            }
        }

        let edges_map = self.edges.lock().unwrap();
        let result_edges = result_edge_ids.iter().filter_map(|id| edges_map.get(id).cloned()).collect();
        Ok(result_edges)
    }

    async fn get_nodes_by_type_and_property(
        &self,
        node_type: NodeType,
        property_key: &str,
        property_value: &Value,
    ) -> Result<Vec<Node>, String> {
        let nodes_map = self.nodes.lock().unwrap();
        let results = nodes_map.values()
            .filter(|n| n.node_type == node_type)
            .filter(|n| n.properties.get(property_key) == Some(property_value))
            .cloned()
            .collect();
        Ok(results)
    }

    async fn query_cypher(&self, _query: &str, _params: Option<HashMap<String, Value>>) -> Result<Vec<HashMap<String, Value>>, String> {
        // InMemoryKnowledgeStore does not support Cypher. This is a placeholder.
        Err("Cypher queries are not supported by InMemoryKnowledgeStore".to_string())
    }

    async fn update_node_properties(&self, node_id: &str, properties_to_update: Value) -> Result<(), String> {
        let mut nodes = self.nodes.lock().unwrap();
        if let Some(node) = nodes.get_mut(node_id) {
            if let Value::Object(update_map) = properties_to_update {
                if let Value::Object(ref mut current_props) = node.properties {
                    for (k, v) in update_map {
                        current_props.insert(k, v);
                    }
                } else {
                     return Err("Node properties are not a JSON object".to_string());
                }
                node.updated_at = Utc::now();
                Ok(())
            } else {
                Err("properties_to_update must be a JSON object".to_string())
            }
        } else {
            Err(format!("Node with id {} not found", node_id))
        }
    }

    async fn update_edge_properties(&self, edge_id: &str, properties_to_update: Value) -> Result<(), String> {
         let mut edges = self.edges.lock().unwrap();
        if let Some(edge) = edges.get_mut(edge_id) {
            if let Value::Object(update_map) = properties_to_update {
                 if let Value::Object(ref mut current_props) = edge.properties {
                    for (k, v) in update_map {
                        current_props.insert(k, v);
                    }
                } else {
                    return Err("Edge properties are not a JSON object".to_string());
                }
                edge.updated_at = Utc::now();
                Ok(())
            } else {
                Err("properties_to_update must be a JSON object".to_string())
            }
        } else {
            Err(format!("Edge with id {} not found", edge_id))
        }
    }

    async fn delete_node(&self, node_id: &str) -> Result<(), String> {
        // First, remove all edges connected to this node
        let edges_to_remove: Vec<String> = {
            let adj_out = self.adj_outgoing.lock().unwrap();
            let adj_in = self.adj_incoming.lock().unwrap();
            let mut ids = HashSet::new();
            if let Some(out_ids) = adj_out.get(node_id) { ids.extend(out_ids.clone()); }
            if let Some(in_ids) = adj_in.get(node_id) { ids.extend(in_ids.clone()); }
            ids.into_iter().collect()
        };

        for edge_id in edges_to_remove {
            self.delete_edge(&edge_id).await?;
        }

        // Then remove the node itself
        if self.nodes.lock().unwrap().remove(node_id).is_none() {
            return Err(format!("Node with id {} not found for deletion", node_id));
        }
        self.adj_outgoing.lock().unwrap().remove(node_id);
        self.adj_incoming.lock().unwrap().remove(node_id);
        Ok(())
    }

    async fn delete_edge(&self, edge_id: &str) -> Result<(), String> {
        let mut edges_map = self.edges.lock().unwrap();
        if let Some(edge) = edges_map.remove(edge_id) {
            if let Some(source_edges) = self.adj_outgoing.lock().unwrap().get_mut(&edge.source_node_id) {
                source_edges.remove(edge_id);
            }
            if let Some(target_edges) = self.adj_incoming.lock().unwrap().get_mut(&edge.target_node_id) {
                target_edges.remove(edge_id);
            }
            Ok(())
        } else {
            Err(format!("Edge with id {} not found for deletion", edge_id))
        }
    }
}
