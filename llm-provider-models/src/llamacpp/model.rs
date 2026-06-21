use serde::{Deserialize, Serialize};

/// Model architecture details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArchitecture {
    /// Supported input modalities (e.g., "text").
    pub input_modalities: Vec<String>,

    /// Supported output modalities (e.g., "text").
    pub output_modalities: Vec<String>,
}

/// Model loading status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    /// Current status value (e.g., "unloaded", "loaded").
    pub value: String,

    /// Server command-line arguments used to load the model.
    pub args: Vec<String>,

    /// Preset configuration string.
    pub preset: String,
}

/// Optional metadata for a model (e.g., parameter counts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMeta {
    /// Vocabulary type identifier.
    pub vocab_type: u64,

    /// Total number of tokens in the vocabulary.
    pub n_vocab: u64,

    /// Maximum context length.
    pub n_ctx: u64,

    /// Context length used during training.
    pub n_ctx_train: u64,

    /// Embedding dimension size.
    pub n_embd: u64,

    /// Total number of model parameters.
    pub n_params: u64,

    /// Model file size in bytes.
    pub size: u64,
}

/// A single model entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Unique identifier for the model.
    pub id: String,

    /// Alternative names for the model.
    #[serde(default)]
    pub aliases: Vec<String>,

    /// Semantic tags for the model.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Object type (e.g., "model").
    pub object: String,

    /// Owner of the model.
    pub owned_by: String,

    /// Unix timestamp of creation.
    pub created: u64,

    /// Current loading status.
    pub status: ModelStatus,

    /// Model architecture details.
    pub architecture: ModelArchitecture,

    /// Whether the model needs to be downloaded.
    pub need_download: bool,

    /// Additional metadata, if available.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ModelMeta>,
}

impl Model {
    pub fn is_loaded(&self) -> bool {
        self.status.value == "loaded"
    }
}

/// Root JSON structure for the model list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListModelsResponse {
    /// List of model entries.
    pub data: Vec<Model>,

    /// Type identifier (e.g., "list").
    pub object: String,
}
