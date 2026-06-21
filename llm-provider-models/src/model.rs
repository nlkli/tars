//! Model listing types returned by the `/v1/models` endpoint.

use serde::Deserialize;

/// A paginated list of available models.
#[derive(Debug, Clone, Deserialize)]
pub struct ListModelsResponse {
    /// The models returned in this page.
    #[serde(default)]
    pub data: Vec<Model>,

    /// Always `"list"`.
    pub object: String,
}

/// Metadata for a single model.
#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    /// Unique model identifier (e.g. `"gpt-4o"`).
    pub id: String,

    /// Unix timestamp when the model was created.
    pub created: u64,

    /// Organization that owns the model.
    pub owned_by: String,

    /// Always `"model"`.
    pub object: String,
}
