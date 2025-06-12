// Telemetry module for Goose: Reasoning Traces, Metrics, etc.

pub mod reasoning_trace;

// Re-export key items for easier access
pub use reasoning_trace::{
    ReasoningTrace,
    DecisionType,
    TraceEmitter,
    InMemoryTraceEmitter,
    // AsyncLogTraceEmitter, // Uncomment when implemented
};
