use serde::{Deserialize, Serialize};

use super::definition::Tool;

// ---------------------------------------------------------------------------
// Helper types
// ---------------------------------------------------------------------------

/// A wrapper carrying only a tool name, used inside named-tool choice variants.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolName {
    pub name: String,
}

impl ToolName {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

// ---------------------------------------------------------------------------
// Mode-only choice
// ---------------------------------------------------------------------------

/// High-level mode for tool invocation, when no specific tool is forced.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    /// The model will not call any tools.
    None,
    /// The model decides whether to call a tool.
    Auto,
    /// The model must call at least one tool.
    Required,
}

// ---------------------------------------------------------------------------
// Allowed-tools choice
// ---------------------------------------------------------------------------

/// Restricts tool calls to the provided subset while applying a mode.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AllowedToolChoice {
    #[serde(rename = "type")]
    pub type_: AllowedToolsType,

    pub mode: AllowedToolsMode,

    pub tools: Vec<Tool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AllowedToolsType {
    #[serde(rename = "allowed_tools")]
    AllowedTools,
}

/// How the model picks from the allowed set.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AllowedToolsMode {
    Auto,
    Required,
}

// ---------------------------------------------------------------------------
// Named function choice
// ---------------------------------------------------------------------------

/// Forces the model to call a specific function by name.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NamedFunctionToolChoice {
    #[serde(rename = "type")]
    pub type_: FunctionToolType,

    pub function: ToolName,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FunctionToolType {
    #[serde(rename = "function")]
    Function,
}

// ---------------------------------------------------------------------------
// Named custom choice
// ---------------------------------------------------------------------------

/// Forces the model to invoke a specific custom tool by name.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NamedCustomToolChoice {
    #[serde(rename = "type")]
    pub type_: CustomToolType,

    pub custom: ToolName,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CustomToolType {
    #[serde(rename = "custom")]
    Custom,
}

// ---------------------------------------------------------------------------
// Top-level union
// ---------------------------------------------------------------------------

/// Specifies which tool(s) the model is allowed or required to call.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Global mode without restricting which tool is called.
    Mode(ToolChoiceMode),

    /// Restricts the callable tools to an explicit subset.
    AllowedTools(AllowedToolChoice),

    /// Forces a specific function call.
    Function(NamedFunctionToolChoice),

    /// Forces a specific custom tool call.
    Custom(NamedCustomToolChoice),
}

impl ToolChoice {
    /// The model will not call any tools.
    pub fn none() -> Self {
        Self::Mode(ToolChoiceMode::None)
    }

    /// The model freely decides whether to call a tool.
    pub fn auto() -> Self {
        Self::Mode(ToolChoiceMode::Auto)
    }

    /// The model must call at least one tool.
    pub fn required() -> Self {
        Self::Mode(ToolChoiceMode::Required)
    }

    /// Restrict tool calls to `tools`, applying the given `mode`.
    pub fn allowed_tools(mode: AllowedToolsMode, tools: Vec<Tool>) -> Self {
        Self::AllowedTools(AllowedToolChoice {
            type_: AllowedToolsType::AllowedTools,
            mode,
            tools,
        })
    }

    /// Force the model to call the function named `name`.
    pub fn function(name: impl Into<String>) -> Self {
        Self::Function(NamedFunctionToolChoice {
            type_: FunctionToolType::Function,
            function: ToolName::new(name),
        })
    }

    /// Force the model to invoke the custom tool named `name`.
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(NamedCustomToolChoice {
            type_: CustomToolType::Custom,
            custom: ToolName::new(name),
        })
    }
}

// ---------------------------------------------------------------------------
// Backward-compat type aliases
// ---------------------------------------------------------------------------

pub type ChatCompletionToolChoice = ToolChoice;
pub type ChatCompletionAllowedToolChoice = AllowedToolChoice;
pub type ChatCompletionNamedFunctionToolChoice = NamedFunctionToolChoice;
pub type ChatCompletionNamedCustomToolChoice = NamedCustomToolChoice;
