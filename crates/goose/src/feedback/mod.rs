// Feedback module for Goose

pub mod store;

// Re-export key items
pub use store::{
    FeedbackEntry,
    FeedbackSource,
    FeedbackStoreProvider,
    InMemoryFeedbackStore,
};
