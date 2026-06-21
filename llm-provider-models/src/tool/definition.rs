use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Function tool
// ---------------------------------------------------------------------------

/// JSON Schema definition for a callable function.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionDefinition {
    /// Name the model must use when invoking this function.
    pub name: String,

    /// Human-readable description that helps the model decide when to call this
    /// function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// JSON Schema object describing the function's parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,

    /// When `true`, the model must produce output that strictly conforms to
    /// `parameters`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

impl FunctionDefinition {
    /// Create a minimal function definition with only a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            parameters: None,
            strict: None,
        }
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the JSON Schema parameter definition.
    pub fn parameters(mut self, params: Value) -> Self {
        self.parameters = Some(params);
        self
    }

    /// Enable strict output mode.
    pub fn strict(mut self) -> Self {
        self.strict = Some(true);
        self
    }
}

// ---------------------------------------------------------------------------
// Custom tool
// ---------------------------------------------------------------------------

/// Definition for a non-function ("custom") tool.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomToolDefinition {
    /// Unique name for this tool.
    pub name: String,

    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Output format specification for the tool's response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<CustomToolFormat>,
}

impl CustomToolDefinition {
    /// Create a minimal custom tool definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            format: None,
        }
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the output format.
    pub fn format(mut self, format: CustomToolFormat) -> Self {
        self.format = Some(format);
        self
    }
}

/// Output format for a [`CustomToolDefinition`].
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CustomToolFormat {
    /// Unstructured text output.
    #[serde(rename = "text")]
    Text,

    /// Output constrained by a formal grammar.
    #[serde(rename = "grammar")]
    Grammar { grammar: GrammarDefinition },
}

/// A formal grammar that constrains custom tool output.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GrammarDefinition {
    /// The grammar source text.
    pub definition: String,

    /// The grammar formalism (e.g. `"EBNF"`, `"ABNF"`).
    pub syntax: String,
}

// ---------------------------------------------------------------------------
// Top-level tool union
// ---------------------------------------------------------------------------

/// A tool that can be registered with a completion request.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Tool {
    /// A callable function described by a JSON Schema.
    #[serde(rename = "function")]
    Function { function: FunctionDefinition },

    /// A provider-specific or application-defined custom tool.
    #[serde(rename = "custom")]
    Custom { custom: CustomToolDefinition },
}

impl Tool {
    /// Wrap a [`FunctionDefinition`] as a [`Tool::Function`].
    pub fn function(fd: FunctionDefinition) -> Self {
        Self::Function { function: fd }
    }

    /// Wrap a [`CustomToolDefinition`] as a [`Tool::Custom`].
    pub fn custom(ctd: CustomToolDefinition) -> Self {
        Self::Custom { custom: ctd }
    }
}

// ---------------------------------------------------------------------------
// Backward-compat type alias
// ---------------------------------------------------------------------------

/// Alias kept for backward compatibility. Prefer [`Tool`].
pub type ChatCompletionTool = Tool;
