// Prompt Variants module for Goose: Storing, managing, and selecting prompt variations.

pub mod manager;

// Re-export key items
pub use manager::{
    PromptVariant,
    PromptVariantProvider,
    InMemoryPromptVariantProvider,
};
