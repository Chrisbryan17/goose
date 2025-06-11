## Goose Framework: Codebase Overview, Internals, Features, and Agentic Nature

Goose is a **custom-built agentic framework** designed to automate tasks by leveraging Large Language Models (LLMs) in conjunction with a versatile tool system. It is not merely an LLM wrapper but a comprehensive platform for building and running AI agents.

**Internals & How it Works:**

At its core is the **`Agent`**, which orchestrates the entire workflow. The process typically follows this loop:

1.  **Dynamic Prompt Construction**: The `PromptManager` assembles a system prompt. This prompt is dynamic, incorporating conversation history, general instructions, details about currently loaded `Extensions` (including their specific instructions and available tools), the current date/time, and operational parameters like the `GOOSE_MODE`. Templates (often model-specific) are embedded in the binary and rendered with this context.
2.  **LLM Interaction**: The `Agent` uses a `Provider` (an abstraction for different LLM APIs like OpenAI, Anthropic, etc.) to send the prompt and message history to the chosen LLM.
3.  **Response & Tool Call Handling**: The LLM's response is processed. If it's a textual answer, it's relayed to the user. If the LLM requests tool usage, the `Agent` parses these `ToolCall` requests.
4.  **Tool Orchestration & Extensibility**:
    *   **Extensions**: Goose's capabilities are primarily extended through `Extensions`. These are often separate processes (MCP clients) managed by the `ExtensionManager`. The `ExtensionManager` loads configurations (`ExtensionConfig`), initializes connections to these extensions, and discovers the tools and prompts they provide. Tool names are prefixed (e.g., `extension_name__tool_name`) for clarity.
    *   **Tool Definition**: Tools are defined with a name, description, JSON schema for inputs, and behavioral `annotations` (e.g., `read_only_hint`).
    *   **Dispatch**: The `Agent` dispatches tool calls. Platform-specific tools (like managing extensions or searching for tools via the `RouterToolSelector`) are handled directly. Others are routed via the `ExtensionManager` to the correct extension. Frontend tools are designated for client-side execution.
    *   **Monitoring**: A `ToolMonitor` helps prevent an LLM from getting stuck in repetitive tool call loops by limiting consecutive identical calls.
    *   **Discovery**: An optional `RouterToolSelector` (e.g., `VectorToolSelector`) can use semantic search to help the LLM find relevant tools from a large set.
5.  **Results & Iteration**: Tool results (or errors) are sent back to the `Agent`, which then typically formulates a new prompt including these results and continues the loop with the LLM until the task is complete.

**Core Features:**

*   **Task Automation**: Achieved via the iterative LLM-tool interaction loop described above.
*   **Planning**: The framework encourages the LLM to create and follow plans for complex, multi-step tasks, moving beyond simple Q&A.
*   **Extensibility**: New tools and capabilities are added through the `Extension` system (MCP clients), making Goose adaptable to diverse tasks.
*   **Permission Model**: The `GOOSE_MODE` (e.g., "chat", "auto", "approve", "smart_approve") provides granular control over the agent's autonomy in using tools.
*   **Error Handling & Self-Correction**: Tool errors are fed back to the LLM, allowing it to attempt different approaches. The `ToolMonitor` adds a layer of robustness.
*   **Streaming**: LLM responses, tool execution requests, and intermediate tool notifications are streamed for a more interactive user experience.
*   **Recipe Generation**: A `create_recipe` function allows the system to generate a summary of a successful task execution (instructions, activities, extensions used) from the conversation history.
*   **Configuration**: A robust `Config` system manages global settings, provider API keys (with keyring support for secrets), and operational parameters. `SessionConfig` tracks session-specific details like working directory and ID, which is used for metrics like token usage.

**Agentic Framework Nature:**

Goose is definitively an agentic framework because it provides:

*   A **sophisticated control loop** (`Agent::reply`) that manages the agent's lifecycle beyond single LLM calls.
*   **Advanced tool definition, orchestration, and extensibility** that allow the agent to interact with its environment and acquire new skills.
*   Mechanisms that **encourage planning and goal-oriented behavior**.
*   **State management** for conversations, configurations, and session data.
*   **Internal logic** for permissions, error handling, and dynamic behavior modification.

This distinguishes it significantly from a simple LLM wrapper, which would primarily focus on API abstraction rather than providing the comprehensive infrastructure for building and running autonomous or semi-autonomous agents.
