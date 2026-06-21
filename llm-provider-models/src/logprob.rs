//! Log-probability types returned when `logprobs: true` is set on a request.

use serde::{Deserialize, Serialize};

/// Log-probability data for all output tokens in a [`ChatCompletionChoice`](crate::completion::ChatCompletionChoice).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionLogprobs {
    /// Per-token log probabilities for the content tokens.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<TokenLogprob>>,

    /// Per-token log probabilities for refusal tokens, when the model refused.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<TokenLogprob>>,
}

/// Log-probability information for a single output token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLogprob {
    /// The token string as decoded text.
    pub token: String,

    /// Raw UTF-8 bytes of the token (useful when the text representation is
    /// ambiguous or lossy).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,

    /// Natural log probability of this token.
    pub logprob: f32,

    /// The top alternative tokens and their log probabilities at this position.
    pub top_logprobs: Vec<TopLogprob>,
}

/// A single alternative token considered at a given position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLogprob {
    /// The alternative token as decoded text.
    pub token: String,

    /// Raw UTF-8 bytes of the alternative token.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,

    /// Natural log probability of this alternative token.
    pub logprob: f32,
}

// ---------------------------------------------------------------------------
// Backward-compat type aliases
// ---------------------------------------------------------------------------

pub type ChatCompletionTokenLogprob = TokenLogprob;
