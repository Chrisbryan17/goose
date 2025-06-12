# Goose Framework: Advanced Features Implementation Plan

## Introduction
This document provides a detailed, phased implementation plan for the advanced features outlined in `ADVANCED_GOOSE_FEATURES.md`. The goal is to enhance the Goose agentic framework systematically, building upon its current codebase to introduce more sophisticated reasoning, learning, and operational capabilities. This plan aims to guide development by breaking down complex features into manageable phases.

## Guiding Principles for Implementation
- **Phased Approach**: Implement features in logical phases to allow for manageable development cycles, iterative testing, and incremental value delivery.
- **Prioritize Foundational Elements**: Focus early on core components (like telemetry and basic KB structure) that enable or support multiple advanced features.
- **Modularity & Clear Interfaces**: Design new components with well-defined responsibilities and interfaces (traits) to ensure loose coupling and testability.
- **Integrated Telemetry**: Incorporate `ReasoningTrace` logging and `FeedbackStore` interactions from the outset of each new feature's development to ensure observability and data for future learning.
- **Configuration-Driven**: Make new features and significant behavioral changes toggleable and configurable via the existing `Config` system to allow for experimentation and controlled rollout.
- **Iterative Refinement**: Recognize that designs may evolve as implementation proceeds and new insights are gained.

## Phase 1: Foundational Advanced Reasoning & Learning Setup

This phase focuses on establishing the basic infrastructure for telemetry, feedback, knowledge storage, and prompt management, which are prerequisites for most advanced capabilities.

*   **Re-analyze Core Goose Modules**:
    *   **Summary of Findings**:
        *   `Agent::reply` is the central loop where most new logic (tracing, feedback hooks, planning state checks, KB queries, prompt variant selection) will be integrated. It's complex and will require careful, incremental modification to maintain clarity. Significant parts will need to become `async`.
        *   `PromptManager` is the natural place for `PromptVariantManager` integration and for injecting KB-derived context into prompts. Its methods will need to become `async`.
        *   `ExtensionManager` will interface with the `KnowledgeStore` to register tool capabilities. Its methods may also need to become `async`.
        *   `Provider` implementations need modification to support confidence scoring.
        *   Overall, a key challenge is managing the transition from largely synchronous operations in some core paths to `async` operations required by new I/O-bound services (DBs, etc.).

*   **Design and Integrate `ReasoningTrace`**:
    *   **Objective**: Establish a system for detailed logging of the agent's decision-making processes to enable debugging, explainability, and future learning.
    *   **Affected Modules**: `Agent` (main loop, tool dispatch methods like `dispatch_tool_call`, LLM interaction logic like `generate_response_from_provider`), `Provider` implementations.
    *   **New Components**:
        *   `crates/goose/src/telemetry/reasoning_trace.rs`: Define `ReasoningTrace` struct and `DecisionType` enum as previously specified.
        *   `TraceEmitter` trait: Define `async fn emit_trace(&self, trace: ReasoningTrace) -> Result<(), String>;`.
        *   Initial implementations:
            *   `InMemoryTraceEmitter`: Stores traces in a `Arc<Mutex<Vec<ReasoningTrace>>>` for testing.
            *   `AsyncLogFileTraceEmitter`: Writes traces as structured JSON lines to a configured log file. Each session could have its own trace log file, or logs could be aggregated.
    *   **Integration**:
        *   Add `trace_emitter: Arc<dyn TraceEmitter>` to the `Agent` struct, initialized in `Agent::new()`.
        *   Modify `Agent::reply` and its helper methods to create `ReasoningTrace` instances at key decision points (e.g., before LLM call with `PromptFinalization` type, after LLM response with `LlmResponseProcessing`, before tool dispatch with `ToolSelection`/`ToolCallDispatch`, after tool result with `ToolResponseProcessing`).
        *   A unique `session_id` (from `SessionConfig`) must be part of each trace. `parent_trace_id` should be used to link sub-decisions.
        *   Configuration: Add `config.yaml` options for trace logging level (e.g., "debug", "info"), backend type ("memory", "logfile"), and log file path.

*   **Design and Integrate `FeedbackStore`**:
    *   **Objective**: Create a persistent store for structured user feedback, tool execution outcomes, and agent observations to inform learning and evaluation.
    *   **Affected Modules**: `Agent`, potentially new API endpoints if Goose is run as a server.
    *   **New Components**:
        *   `crates/goose/src/feedback/store.rs`: Define `FeedbackEntry` struct and `FeedbackSource` enum as previously specified.
        *   `FeedbackStoreProvider` trait: Define methods like `async fn store_feedback(&self, entry: FeedbackEntry) -> Result<(), String>;`, `async fn get_feedback_for_trace(&self, trace_id: &str) -> Result<Vec<FeedbackEntry>, String>;`.
        *   Initial implementations:
            *   `InMemoryFeedbackStore`: For testing.
            *   `SqliteFeedbackStore`: Uses `rusqlite` (with `tokio-rusqlite` for async) to store feedback in a local SQLite database file. Define table schema based on `FeedbackEntry`.
    *   **Integration**:
        *   Add `feedback_store: Arc<dyn FeedbackStoreProvider>` to `Agent` struct.
        *   The application embedding Goose (CLI, server) will need to provide mechanisms for users to submit feedback (e.g., commands, UI elements that call new `Agent` methods like `agent.submit_user_feedback(...)`).
        *   `Agent` itself will log feedback:
            *   After tool execution: Log success/failure, execution time as `FeedbackSource::AgentObservation` or `ToolInternalError` if error is structured. `related_trace_id` links to the tool call trace.
            *   Task completion: `Agent` logs task success/failure (e.g., based on `submit_subtask_report` outcome or LLM confirmation) as `FeedbackSource::AgentObservation`.

*   **Design and Integrate `KnowledgeStore` (Graph DB - Initial Setup)**:
    *   **Objective**: Lay the groundwork for a persistent graph knowledge base to store and query information about entities, tools, sessions, and their relationships.
    *   **Affected Modules**: `Agent`, `ExtensionManager`.
    *   **New Crate**: `crates/goose_kb` containing:
        *   `graph_schema.rs` (or `lib.rs`): Defines `Node`, `Edge`, `NodeType`, `EdgeType` structs/enums as previously specified.
        *   `KnowledgeStoreProvider` trait: Defines methods like `async fn add_node(&self, node: &Node) -> Result<(), String>;`, `async fn add_edge(&self, edge: &Edge) -> Result<(), String>;`, `async fn get_node_by_id(&self, node_id: &str) -> Result<Option<Node>, String>;`, `async fn query_cypher(...)` (if using Cypher-based DB).
        *   Initial implementations:
            *   `InMemoryKnowledgeStore`: Basic in-memory graph for testing (as designed in previous step).
            *   `LocalGraphDbProvider`: Wrapper for a local Dockerized graph database (e.g., Neo4j, Memgraph). This provider would translate trait calls into Cypher queries or specific DB client library calls. Requires clear setup instructions (Docker image, connection string in `config.yaml`).
    *   **Integration**:
        *   Add `knowledge_store: Arc<dyn KnowledgeStoreProvider>` to `Agent` struct.
        *   **Initial Population**:
            *   On session start: Create/retrieve `Node` for current `Agent`, `User` (if identifiable), and `Session`. Link them (e.g., `(User)-[:INITIATED]->(Session)`).
            *   When `ExtensionManager` loads tools (e.g., in `Agent::add_extension` or a sync step after init): For each tool, create/update a `Tool` node in the KB and link it to the `Agent` node (e.g., `(Agent)-[:HAS_CAPABILITY]->(Tool)`).
        *   **Basic Queries**: Initially, queries will be simple (e.g., "Fetch all tools known by this agent"). Usage will expand in later phases.

*   **Design and Integrate `PromptVariantManager` (Basic Structure)**:
    *   **Objective**: Establish a system to manage different versions of prompt templates, allowing for future optimization and A/B testing.
    *   **Affected Modules**: `PromptManager`.
    *   **New Components**:
        *   `crates/goose/src/prompt_variants/manager.rs`: Define `PromptVariant` struct as previously specified.
        *   `PromptVariantProvider` trait: Define methods like `async fn get_active_variant(&self, prompt_type_key: &str) -> Result<Option<PromptVariant>, String>;`, `async fn store_variant(&self, variant: &PromptVariant) -> Result<(), String>;`, `async fn update_variant_metrics(...)`.
        *   Initial implementations:
            *   `InMemoryPromptVariantProvider`: For testing.
            *   `JsonFilePromptVariantProvider`: Stores prompt variants in a JSON file. `get_active_variant` might load all, filter by `prompt_type_key` and `is_active`, then select the one with the highest `version`.
    *   **Integration**:
        *   Add `prompt_variant_provider: Arc<dyn PromptVariantProvider>` to `PromptManager` struct (or pass it into methods).
        *   `PromptManager::build_system_prompt` (and other prompt-generating methods) will:
            1.  Determine the `prompt_type_key` (e.g., "SystemPrompt_Main", "StrategicPlanningPrompt").
            2.  Call `prompt_variant_provider.get_active_variant(key).await`.
            3.  Use the `template_text` from the returned `PromptVariant` for rendering.
            4.  If no variant is found, fall back to a hardcoded default or an error.
        *   No learning/selection algorithm in Phase 1; it just serves the "active" (e.g., latest) variant.

## Phase 2: Implementing Initial Advanced Reasoning & Learning Capabilities

This phase builds upon the foundational elements from Phase 1 to introduce initial versions of advanced features.

*   **Implement Basic Hierarchical Planning**:
    *   **Objective**: Enable the agent to break down complex user goals into a hierarchy of strategic goals, tactical plans, and operational steps.
    *   **Affected Modules**: `Agent` (state management, `reply` loop logic), `PromptManager`.
    *   **New Components**:
        *   `crates/goose/src/planning/hierarchical.rs`: Use `StrategicGoal`, `TacticalPlan`, `OperationalStep`, and `PlanStatus` structs defined in Phase 1.
        *   Add `active_strategic_goal: Arc<Mutex<Option<StrategicGoal>>>`, `active_tactical_plan: Arc<Mutex<Option<TacticalPlan>>>`, `operational_steps_queue: Arc<Mutex<VecDeque<OperationalStep>>>` to `Agent`'s state.
    *   **Integration**:
        *   **`PromptManager`**: Develop new prompt templates (`strategic_planning.md`, `tactical_planning.md`, `operational_planning.md`). `build_system_prompt` (or a new method `get_planning_prompt(current_state)`) selects the appropriate template based on the agent's current planning state (e.g., no goal, goal but no tactical plan, etc.).
        *   **`Agent::reply` Loop**:
            1.  If no `active_strategic_goal`, use user input and strategic planning prompt to generate and store one.
            2.  If goal exists but no `active_tactical_plan`, use goal and tactical planning prompt to generate/select and store one.
            3.  If tactical plan exists but queue is empty, use tactical plan and operational planning prompt to populate `operational_steps_queue`.
            4.  Dequeue and execute `OperationalStep`s:
                *   If tool-based, use existing tool dispatch logic. Update step status based on `ToolCallResult`.
                *   If human action, yield message to user and await confirmation.
            5.  Log creation, selection, and status updates of plan elements using `ReasoningTrace`.
        *   **Error Handling**: If a step fails, mark its status. Re-prompt LLM for replanning at the current level (e.g., new operational steps) or escalate to higher level if current plan is unrecoverable. Log `KnowledgeGapEntry` if failure is due to missing info.

*   **Implement Confidence Scoring (LLM Self-Assessment)**:
    *   **Objective**: Capture the LLM's self-assessed confidence in its responses for logging and potential future use in decision-making.
    *   **Affected Modules**: `Provider` trait and all its implementations (e.g., `OpenAIProvider`), `Agent`.
    *   **Integration**:
        *   Modify `Provider::complete()` to return `Result<ProviderCompletionResponse, ProviderError>`, where `ProviderCompletionResponse` includes `message: Message`, `usage: ProviderUsage`, and `self_assessed_confidence: Option<f32>`.
        *   **Provider Implementations**: Adapt prompt engineering to instruct LLM to output its confidence (e.g., "Confidence: X.X" on a new line). Parse this value and exclude it from the user-facing message. If provider APIs offer direct confidence metrics (e.g., logprobs for selected tokens), these could be used/mapped.
        *   **`Agent`**: When `generate_response_from_provider` (internal to `Agent`) gets the `ProviderCompletionResponse`, it extracts `self_assessed_confidence`.
        *   This confidence score is then logged in the `ReasoningTrace` associated with the `LlmResponseProcessing` or `JustificationGeneration` decision type.
        *   **Initial Usage**: Primarily for logging. A low score could trigger a `tracing::warn!`. More advanced handling (e.g., automatic re-prompting or user alerts) is for later phases.

*   **Implement Basic Knowledge Gap Identification**:
    *   **Objective**: Allow the agent to identify and log gaps in its knowledge or capabilities during task execution.
    *   **Affected Modules**: `Agent`, `KnowledgeStoreProvider` (from `goose_kb`).
    *   **New Components**:
        *   `crates/goose_kb/src/knowledge_gap.rs`: Use `KnowledgeGapEntry` struct and `KnowledgeGapStatus` enum defined in Phase 1.
    *   **Integration**:
        *   **Trigger Points in `Agent::reply`**:
            1.  **Low LLM Confidence**: If `self_assessed_confidence` from `ProviderCompletionResponse` is below a configurable threshold (e.g., 0.5) for a critical response (e.g., a plan step, a factual answer).
            2.  **Explicit LLM Statement**: Basic keyword spotting in LLM responses (e.g., "I need more information about X", "I am unsure how to proceed with Y"). This is a heuristic.
            3.  **Tool Failure**: If a tool fails with an error that suggests missing information or prerequisites (e.g., file not found, invalid parameters due to misunderstanding).
        *   **`KnowledgeGapEntry` Creation**: `Agent` constructs a `KnowledgeGapEntry` with `description_by_llm_or_agent`, `related_trace_id` (linking to the trace where the gap was identified), `session_id`, and sets status to `Open`.
        *   **Storage**: Add `async fn store_knowledge_gap(&self, gap: &KnowledgeGapEntry) -> Result<(), String>;` (and query methods) to `KnowledgeStoreProvider` trait. Implement in `InMemoryKnowledgeStore` and `LocalGraphDbProvider`.
        *   **No Automated Resolution in Phase 2**: Gaps are logged. Agent might optionally inform the user (e.g., "I've identified a knowledge gap regarding X and have logged it for review.").

*   **Implement `KnowledgeExtractionService` (Basic NER/RE)**:
    *   **Objective**: Enable the agent to extract structured information (entities and simple relationships) from textual content generated by tools or users.
    *   **Affected Modules**: `Agent`, `KnowledgeStoreProvider`.
    *   **New Components**:
        *   `crates/goose_kb/src/extraction.rs`: Define `KnowledgeExtractionServiceProvider` trait and `LlmKnowledgeExtractor` implementation as specified in Phase 1. `ExtractionContext` struct to pass metadata.
        *   The `LlmKnowledgeExtractor` will require an `Arc<dyn Provider>` to make LLM calls. This implies careful dependency management if `goose_kb` is a separate crate (it might be better for `KnowledgeExtractionService` to be part of the main `goose` crate or for `Provider` to be in `goose_core`).
    *   **Integration**:
        *   `Agent` obtains an instance of `Arc<dyn KnowledgeExtractionServiceProvider>`.
        *   **Invocation**: After a tool returns significant textual content (e.g., from a web search, file read) or after a detailed user message, the `Agent` can invoke `extractor.extract_from_text(&text_content, &extraction_context).await`.
        *   **Prompting for Extraction**: `LlmKnowledgeExtractor` sends a prompt to its LLM asking it to identify entities (with types like Person, File, Tool, Concept) and simple relationships (e.g., Uses, Mentions, RelatedTo), requesting output in a specific JSON format matching simplified `Node` and `Edge` structures.
        *   **Storing Extractions**: The parsed `Node` and `Edge` objects from the extractor are then added to the `KnowledgeStore` via `knowledge_store.add_node()` and `knowledge_store.add_edge()`. Each extracted piece of information should have its `source_document_uri` and `related_trace_id` recorded in its properties.
    *   **Error Handling**: Log errors from LLM calls or JSON parsing during extraction. Extracted data is treated as provisional and may have confidence scores if the LLM provides them.

*   **Implement Cross-Session Learning Repository (Initial Version - Manual/Simple Aggregation)**:
    *   **Objective**: Create a persistent store for sharing learned artifacts like aggregated tool performance and designated best prompt variants across sessions.
    *   **Affected Modules**: `Agent` (for querying), `PromptManager` (for querying).
    *   **New Components**:
        *   `crates/goose/src/learning/cross_session.rs` (new module):
            *   Define structs: `AggregatedToolMetrics { tool_name: String, total_runs: u64, success_count: u64, avg_duration_ms: Option<f64>, last_updated_utc: DateTime<Utc> }`.
            *   Define `CrossSessionLearningProvider` trait with methods like `async fn get_aggregated_tool_metrics(&self, tool_name: &str) -> Result<Option<AggregatedToolMetrics>, String>;`, `async fn update_aggregated_tool_metrics(...)`, `async fn get_best_prompt_variant_id(&self, prompt_type_key: &str) -> Result<Option<String>, String>;`, `async fn store_best_prompt_variant_id(...)`.
        *   Initial backend: SQLite tables (`aggregated_tool_performance`, `best_prompt_variants`).
    *   **Initial Update Mechanism**:
        *   **Tool Performance**: No automated aggregation in Phase 2. A separate script (run manually) would analyze the `FeedbackStore` (once it has enough data) to calculate and populate/update the `AggregatedToolPerformance` table.
        *   **Prompt Variants**: An administrator or script would manually designate a `variant_id` as "best" for a given `prompt_type_key` in the `best_prompt_variants` table.
    *   **Integration**:
        *   `Agent` (or its `ToolMonitor`/`ToolRouter`) can query `get_aggregated_tool_metrics` to potentially influence tool selection (e.g., log a warning if a historically unreliable tool is chosen, or slightly prefer tools with high success rates if multiple options exist).
        *   `PromptManager` (via its `PromptVariantProvider` or directly) can query `get_best_prompt_variant_id`. If a "best" ID is found, it then fetches that specific variant from its primary `PromptVariantProvider`. This allows global promotion of effective prompts.

## Phase 3: Enhancing Self-Understanding & Dynamic Tool Management
(This phase and subsequent ones will build heavily on Phase 1 & 2 components.)

*   **Implement Metacognitive Prompting**:
    *   **Objective**: Enable the agent to reflect on its own reasoning processes and improve decision quality.
    *   **Affected Modules**: `PromptManager`, `Agent` (`reply` loop, planning logic).
    *   **Integration**:
        *   `PromptManager` will include a library of "metacognitive snippets" (e.g., "List pros and cons for the top 3 tool choices," "Assess confidence in the current plan and identify key uncertainties").
        *   `Agent::reply` logic, at specific decision points (e.g., after initial tool selection by LLM, after plan generation), will instruct `PromptManager` to inject a relevant metacognitive snippet into the next prompt to the LLM.
        *   The LLM's response to the metacognitive part of the prompt will be logged in `ReasoningTrace.justification_llm_response`.
        *   Initially, this justification is for logging. Later, it could directly influence plan/tool choice or trigger `KnowledgeGap` identification.

*   **Design and Integrate `ExternalToolRegistryManager` & `ToolSpecificationParser`**:
    *   **Objective**: Enable the agent to discover new tools from external sources like OpenAPI specifications.
    *   **Affected Modules**: `ExtensionManager`, potentially a new `Agent` capability.
    *   **New Components**:
        *   `crates/goose/src/tool_discovery/mod.rs`:
            *   `ExternalToolRegistryManager`: Configured with URLs to known registries (e.g., a list of OpenAPI JSON files). Has methods like `async fn scan_for_new_tools() -> Vec<ToolCandidate>`.
            *   `ToolCandidate`: Struct holding parsed name, description, server URL, basic schema info.
            *   `ToolSpecificationParser`: Takes an OpenAPI spec URL/content, parses it (using crates like `openapi`), and extracts a structured `ToolDefinition` (more detailed than `ToolCandidate`, aligning with `mcp_core::Tool` but including endpoint details).
            *   `CandidateToolEvaluator` (conceptual for now): An LLM-prompted service to assess relevance/safety of a parsed `ToolDefinition`.
    *   **Integration**:
        *   `ExternalToolRegistryManager` runs periodically or on command.
        *   New `ToolDefinition`s (after parsing and evaluation) are presented to a human operator for approval in Phase 3.
        *   Approved tools would require manual or semi-automated creation of an `ExtensionConfig` (e.g., for a generic HTTP tool wrapper extension) to be loaded by `ExtensionManager`.

*   **Implement `ToolParameterOptimizer` (Proof of Concept)**:
    *   **Objective**: Demonstrate learning optimal default values for a *single, specific, safe* tool parameter.
    *   **Affected Modules**: `FeedbackStoreProvider`, `CrossSessionLearningProvider`, a specific `Tool`'s interaction logic.
    *   **Integration**:
        *   Select one tool with a tunable, numeric parameter (e.g., a `web_search` tool with `max_results`).
        *   Log `(parameter_value_used, task_success_metric)` to `FeedbackStore` for this tool.
        *   A manual script analyzes this data from `FeedbackStore` to find a parameter value that correlates with better outcomes for `web_search`.
        *   This "optimal" value is stored via `CrossSessionLearningProvider.store_tool_parameter_override("web_search", "max_results", optimal_value)`.
        *   The `web_search` tool's logic would then query `CrossSessionLearningProvider` for this override at runtime.
        *   No automated tuning loop in this phase; focus is on data path and manual analysis.

## Phase 4: Advanced Reasoning - Part 1 (Uncertainty, Causality, Basic Collaboration)
*   **Implement Uncertainty Modeling in Planning**:
    *   **Objective**: Allow plans to represent and reason about uncertainty.
    *   **Affected Modules**: `planning::hierarchical` structs, `Agent` planning logic, `PromptManager`.
    *   **Integration**:
        *   Add `confidence_score: Option<f32>` to `Condition`s in planning structs.
        *   `OperationalStep` can have `possible_outcomes: Option<Vec<{description: String, probability: f32}>>`.
        *   `PromptManager`: Prompts for planning should ask LLM to estimate confidence in preconditions being met or likelihood of step success.
        *   `Agent`: When executing plans, if confidence is low for a critical step, it might insert a verification step (use a tool to check a condition) or choose more robust (but perhaps less efficient) paths.
        *   New (conceptual) `UncertaintyManager`: Could track belief states based on LLM assessments and tool outcomes, updating confidence scores. For Phase 4, this is likely just direct use of scores by `Agent`.

*   **Implement `CausalModelManager` (Initial Structure & Manual Population)**:
    *   **Objective**: Begin storing and using causal relationships.
    *   **Affected Modules**: `Agent`, `goose_kb`.
    *   **New Components**:
        *   `crates/goose_kb/src/causal.rs`: Define `CausalNode` (event/state variable) and `CausalEdge` (probabilistic/functional link) structs.
        *   `CausalModelManager` (within `goose_kb`): Provides methods to add/query these relationships. Storage within the main `KnowledgeStoreProvider` (e.g., specific node/edge types).
    *   **Integration**:
        *   For a very limited domain (e.g., debugging a specific software interaction), manually populate a few causal links via a script (e.g., "High CPU Usage" CAUSES "Slow Response Time").
        *   `Agent`: If a problem occurs (e.g., tool failure), it can query `CausalModelManager` for potential causes of observed symptoms to include in its diagnostic prompts to the LLM.

*   **Implement Inter-Agent Communication (Basic Handshake & Task Offer)**:
    *   **Objective**: Allow one Goose agent to offer a simple, self-contained task to another.
    *   **Affected Modules**: `Agent`.
    *   **New Components**:
        *   `crates/goose/src/multi_agent/communication.rs`: Define basic message schemas (e.g., `SimpleTaskOffer { task_id, description, input_params }`, `SimpleTaskResult { task_id, output, error }`).
        *   Each `Agent` needs a simple network listener (e.g., HTTP endpoint or basic TCP socket) and sender. Agent addresses configured manually.
    *   **Integration**:
        *   Add a new internal tool to `Agent`: `delegate_simple_task(target_agent_address: String, task_description: String, input_params: Value)`.
        *   When called, this tool sends `SimpleTaskOffer` to the target.
        *   The receiving agent's listener, upon getting an offer, could use its LLM to decide if it can do the task (very basic) and then execute it as if it were a user request, returning `SimpleTaskResult`.
        *   No complex brokering, discovery, or shared knowledge in this phase.

## Phase 5: Advanced Learning & Tooling - Part 1 (Automated Learning, Tool Scaffolding, Basic Robustness)
*   **Implement Automated Prompt Optimization (Bandit Algorithm)**:
    *   **Objective**: Enable `PromptVariantProvider` to learn and adapt which prompts are most effective.
    *   **Affected Modules**: `prompt_variants::manager` (implementation of `PromptVariantProvider`), `FeedbackStoreProvider`.
    *   **Integration**:
        *   The chosen `PromptVariantProvider` implementation (e.g., `SqlitePromptVariantProvider`) will implement a multi-armed bandit algorithm (e.g., Epsilon-greedy or UCB1).
        *   When `get_active_variant` is called, it uses the bandit logic to select a `PromptVariant` (balancing exploration/exploitation).
        *   A background process or a call after feedback is collected (`Agent::reply`'s end) will:
            1.  Retrieve feedback for interactions that used specific `variant_id`s (from `ReasoningTrace` which should store `variant_id_used`).
            2.  Calculate a reward score (e.g., based on task success, user rating, efficiency).
            3.  Call `prompt_variant_provider.update_variant_metrics(variant_id, reward_score)` which updates the bandit algorithm's statistics for that variant.

*   **Implement Online Adapter Training Module (Data Collection & Stubbing)**:
    *   **Objective**: Collect data suitable for PEFT and stub out the training infrastructure.
    *   **Affected Modules**: `FeedbackStoreProvider`.
    *   **New Components**:
        *   `crates/goose/src/learning/peft_training/data_collector.rs`: Service to query `FeedbackStore` and `ReasoningTrace`.
    *   **Integration**:
        *   `DataCollectorService`: Filters for high-quality interactions (e.g., successful tasks with positive feedback, user corrections).
        *   Transforms these into `(prompt, completion)` pairs suitable for instruction fine-tuning (e.g., original prompt to LLM, and the LLM's good response; or user's query and agent's final, successful answer).
        *   Saves these pairs to a structured file (e.g., JSONL) or a dedicated "training_data" table.
        *   The actual training module that would consume this data is stubbed out (defines interface but no training logic).

*   **Implement Automated Tool Scaffolding (LLM Prompting)**:
    *   **Objective**: Use LLM to generate boilerplate code for new, simple MCP extensions.
    *   **Affected Modules**: `Agent` (as a user-driven capability).
    *   **New Components**:
        *   `crates/goose/src/tool_scaffolding/mod.rs`: `ToolScaffolder` service.
    *   **Integration**:
        *   User provides a natural language description of a tool's purpose, inputs (with types), and outputs (with types), and potentially an API endpoint if it's a wrapper.
        *   `ToolScaffolder` uses a detailed prompt to instruct an LLM to generate Python code for a simple MCP extension (including `list_tools` and `call_tool` methods, argument parsing from JSON, basic HTTP calls if needed).
        *   The generated code and a candidate `ExtensionConfig` (Stdio type) are saved to files.
        *   Output is clearly marked "EXPERIMENTAL - HUMAN REVIEW REQUIRED." No automated testing or deployment in this phase.

*   **Implement Basic Anomaly Detection (Statistical Rules)**:
    *   **Objective**: Detect simple operational anomalies.
    *   **Affected Modules**: `Agent` (to receive alerts), `ReasoningTrace` (as data source), `FeedbackStore` (as data source).
    *   **New Components**:
        *   `crates/goose/src/runtime_monitoring/basic_detector.rs`: `BasicAnomalyDetector`.
    *   **Integration**:
        *   `BasicAnomalyDetector` runs periodically (e.g., as a separate thread/task spawned by Agent or a cron job).
        *   It queries `ReasoningTrace` and `FeedbackStore` for metrics like:
            *   Tool error rates (count of `ToolResponseProcessing` traces with errors / total calls for a tool).
            *   LLM response times (from `LlmRequestSent` to `LlmResponseProcessing` duration).
        *   If metrics exceed predefined, hardcoded thresholds (e.g., error rate > 25% in last N calls), it logs a `tracing::error!` or sends an internal alert.
        *   No automated recovery yet.

*   **Implement `EthicalCheckModule` (Rule-Based)**:
    *   **Objective**: Introduce basic, non-LLM-based ethical guardrails.
    *   **Affected Modules**: `Agent` (tool dispatch, LLM prompting).
    *   **New Components**:
        *   `crates/goose/src/ethics/guardrails.rs`: `EthicalCheckModule`.
    *   **Integration**:
        *   `EthicalCheckModule` is initialized with a small set of hardcoded rules (e.g., deny list of keywords for prompts/responses, block tool calls to `filesystem::delete_recursive` if path is `/`).
        *   `Agent` calls `ethical_module.check_prompt(prompt_text)` before sending to LLM, and `ethical_module.check_tool_call(tool_name, params)` before dispatch.
        *   If a check fails, the action is blocked, and an error is logged/returned. `ReasoningTrace` logs the veto.

## Phase 6: Advanced Proactive & Contextual Features - Part 1
(Building on earlier phases)

*   **Implement `Event-Driven Goal Proposal` (Basic Event Watcher)**:
    *   **Objective**: Allow agent to proactively suggest tasks based on simple file system events.
    *   **Affected Modules**: `Agent`.
    *   **New Components**:
        *   `crates/goose/src/proactive/file_watcher.rs`: `FileEnvironmentWatcher`. Uses a crate like `notify` to watch a configured directory.
        *   `crates/goose/src/proactive/opportunity_detector.rs`: `BasicOpportunityDetector`.
    *   **Integration**:
        *   `FileEnvironmentWatcher` runs in a background thread, sends `ExternalEvent` (e.g., new file created) via a channel to the `Agent`.
        *   `Agent` passes event to `BasicOpportunityDetector`.
        *   Detector uses simple rules (e.g., "If new .txt file in '~/docs/incoming', propose summarization task").
        *   If a `ProposedGoal` is generated, `Agent` presents it to the user for approval.

*   **Implement `PersistentUserProfileStore` (Basic)**:
    *   **Objective**: Store and retrieve user preferences to personalize agent behavior.
    *   **Affected Modules**: `Agent`, `PromptManager`.
    *   **New Components**:
        *   `crates/goose/src/user_profile/store.rs`: `UserProfile` struct (e.g., `preferred_communication_style: String`, `custom_instructions: Vec<String>`). `UserProfileStoreProvider` trait. Initial impl: `JsonFileUserProfileStore` (one JSON file per user).
    *   **Integration**:
        *   `Agent` loads `UserProfile` at session start (based on `user_id` if available, or a default profile).
        *   `PromptManager` receives `UserProfile` and incorporates `custom_instructions` or `communication_style` into system prompts.
        *   User can set preferences via new agent commands (e.g., `/set_preference style concise`).

*   **Implement `InputPreprocessor` for one Multi-Modal Type (e.g., Image Captioning)**:
    *   **Objective**: Allow agent to understand basic image content by converting it to text.
    *   **Affected Modules**: `Agent`.
    *   **New Components**:
        *   `crates/goose/src/multimodal/image_processor.rs`: `ImageInputPreprocessor`.
    *   **Integration**:
        *   User provides an image URL or local path.
        *   `Agent` calls `image_processor.generate_caption(image_ref)`.
        *   The processor uses an external image captioning library/API (e.g., a Python script called via FFI, or a REST API for an online service). This detail is abstracted from the `Agent`.
        *   The returned text caption is then added to the conversation context for the LLM.

## Phase 7: Future Considerations & Advanced Integration Strategy

*   **Roadmap for More Complex Features**:
    *   **MCTS for Planning**: Would build on Hierarchical Planning (P2) and Uncertainty Modeling (P4), requiring the `UncertaintyManager` to be more fully fleshed out.
    *   **Full Causal Inference Learning**: Requires significant research; would build on `CausalModelManager` (P4) and integrate deeply with `KnowledgeExtractionService` (P2) and `FeedbackStore` (P1).
    *   **Analogical Reasoning**: Needs robust `ReasoningTrace` (P1) and `FeedbackStore` (P1) for case base creation.
    *   **Distributed Planning & Consensus**: Builds on Inter-Agent Communication (P4), requires robust `TaskBroker` and shared `KnowledgeStore` (P1).
    *   **ML-based Anomaly Detection**: Uses data collected by `RuntimeMonitorService` (P5 basic rules) to train more sophisticated models.
    *   **Full Multi-Modal Processing**: Extends `InputPreprocessor` (P6) to more modalities and deeper integration with multi-modal LLMs.
    *   **Social/Emotional Context Analyzer**: A highly advanced feature, building on text processing and potentially requiring dedicated model training. Ethical considerations are paramount.

*   **Testing and Validation Strategy**:
    *   **Unit Tests**: For all new structs, methods, and services (e.g., `TraceEmitter` impls, `FeedbackStoreProvider` impls, planning logic, new service modules).
    *   **Integration Tests**:
        *   Test `Agent`'s ability to log traces correctly with `InMemoryTraceEmitter`.
        *   Test feedback storage and retrieval with `InMemoryFeedbackStore` / `SqliteFeedbackStore`.
        *   Test KB population and basic queries with `InMemoryKnowledgeStore` / `LocalGraphDbProvider`.
        *   Test `PromptManager`'s ability to fetch and use variants from `InMemoryPromptVariantProvider`.
        *   Test planning loop for simple goals.
    *   **End-to-End Scenario Testing**: Define key user scenarios for each phase (e.g., "User provides feedback on a failed tool call, agent logs it," "Agent plans a 3-step task," "Agent uses a prompt variant"). Automate these where possible.
    *   **Human-in-the-Loop Evaluation**: For complex reasoning, planning, and learned behaviors, manual review of `ReasoningTrace`s and agent outputs will be necessary. Subjective evaluation of interaction quality.

*   **Potential Refactoring Needs**:
    *   **`Agent::reply` Monolith**: As more features are added, `Agent::reply` will grow very large. It needs to be broken down into smaller, more manageable private methods or even separate state-handler structs/modules (e.g., `PlanningHandler`, `ExecutionHandler`, `FeedbackHandler`).
    *   **Synchronous Core Paths**: Many existing methods in `Agent`, `PromptManager`, `ExtensionManager` are synchronous. Interactions with new `async` providers (DBs, network services) will force these paths to become `async` all the way up, which is a significant but necessary refactoring. Consider using `tokio::spawn` for truly background tasks where appropriate, but core decision loops will need to be `async`.
    *   **Configuration Loading**: `Config::global()` is convenient but might become a bottleneck. Consider passing `Arc<Config>` around more explicitly or using a more structured dependency injection approach for services needing config.
    *   **Error Handling**: Standardize error types and handling across new modules. Use `thiserror` consistently.

*   **Configuration Management for Advanced Features**:
    *   All new services (`TraceEmitter` backend, `FeedbackStore` type, `KnowledgeStore` connection strings, `PromptVariantProvider` file path) will have their settings in `config.yaml`.
    *   Feature toggles: `enable_reasoning_trace: bool`, `enable_hierarchical_planning: bool`, `enable_confidence_scoring: bool`, etc.
    *   Thresholds for features like knowledge gap identification (`knowledge_gap_confidence_threshold: f32`) will be configurable.
    *   Paths for data storage (SQLite DBs, JSON files, log files) will be configurable.

## Conclusion
Implementing these features in a phased manner will systematically enhance the Goose framework from a reactive agent to a proactive, learning, and deeply introspective system. Each phase builds valuable infrastructure and capabilities, paving the way for increasingly complex and intelligent behaviors. This approach balances ambitious goals with practical, iterative development, ultimately aiming to create a state-of-the-art AI agent that is robust, adaptable, and genuinely useful.
