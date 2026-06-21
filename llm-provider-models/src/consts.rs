/// Content part type identifiers for [`crate::message::ContentPart`].
pub mod content_part {
    pub const TEXT: &str = "text";
    pub const IMAGE_URL: &str = "image_url";
    pub const AUDIO: &str = "input_audio";
    pub const FILE: &str = "file";
    pub const REFUSAL: &str = "refusal";
}

/// Audio format identifiers for [`crate::message::InputAudioData`].
pub mod audio_format {
    pub const MP3: &str = "mp3";
    pub const WAV: &str = "wav";
}

/// Image detail level options for [`crate::message::ImageUrl`].
pub mod image_detail {
    pub const AUTO: &str = "auto";
    pub const LOW: &str = "low";
    pub const HIGH: &str = "high";
}

/// Message role identifiers.
pub mod role {
    pub const DEVELOPER: &str = "developer";
    pub const SYSTEM: &str = "system";
    pub const USER: &str = "user";
    pub const ASSISTANT: &str = "assistant";
    pub const TOOL: &str = "tool";
    pub const FUNCTION: &str = "function";
}

/// Reasoning effort level options.
pub mod reasoning_effort {
    pub const NONE: &str = "none";
    pub const MINIMAL: &str = "minimal";
    pub const LOW: &str = "low";
    pub const MEDIUM: &str = "medium";
    pub const HIGH: &str = "high";
    pub const XHIGH: &str = "xhigh";
}

/// Finish reason values returned in completion choices.
pub mod finish_reason {
    pub const STOP: &str = "stop";
    pub const LENGTH: &str = "length";
    pub const TOOL_CALLS: &str = "tool_calls";
    pub const CONTENT_FILTER: &str = "content_filter";
    pub const FUNCTION_CALL: &str = "function_call";
}

/// Tool call type identifiers.
pub mod tool_call_type {
    pub const FUNCTION: &str = "function";
    pub const CUSTOM: &str = "custom";
}

// ---------------------------------------------------------------------------
// Flat re-exports for backward compatibility with the original `SCREAMING_SNAKE`
// naming convention.
// ---------------------------------------------------------------------------

pub use content_part::AUDIO as CONTENT_PART_AUDIO_TYPE;
pub use content_part::FILE as CONTENT_PART_FILE_TYPE;
pub use content_part::IMAGE_URL as CONTENT_PART_IMAGE_TYPE;
pub use content_part::REFUSAL as CONTENT_PART_REFUSAL_TYPE;
pub use content_part::TEXT as CONTENT_PART_TEXT_TYPE;

pub use audio_format::MP3 as INPUT_AUDIO_DATA_FORMAT_MP3;
pub use audio_format::WAV as INPUT_AUDIO_DATA_FORMAT_WAV;

pub use image_detail::AUTO as IMAGE_URL_DETAIL_AUTO;
pub use image_detail::HIGH as IMAGE_URL_DETAIL_HIGH;
pub use image_detail::LOW as IMAGE_URL_DETAIL_LOW;

pub use role::ASSISTANT as ASSISTANT_ROLE;
pub use role::DEVELOPER as DEVELOPER_ROLE;
pub use role::FUNCTION as FUNCTION_ROLE;
pub use role::SYSTEM as SYSTEM_ROLE;
pub use role::TOOL as TOOL_ROLE;
pub use role::USER as USER_ROLE;

pub use reasoning_effort::HIGH as REASONING_EFFORT_HIGH;
pub use reasoning_effort::LOW as REASONING_EFFORT_LOW;
pub use reasoning_effort::MEDIUM as REASONING_EFFORT_MEDIUM;
pub use reasoning_effort::MINIMAL as REASONING_EFFORT_MINIMAL;
pub use reasoning_effort::NONE as REASONING_EFFORT_NONE;
pub use reasoning_effort::XHIGH as REASONING_EFFORT_XHIGH;

pub use finish_reason::CONTENT_FILTER as FINISH_REASON_CONTENT_FILTER;
pub use finish_reason::FUNCTION_CALL as FINISH_REASON_FUNCTION_CALL;
pub use finish_reason::LENGTH as FINISH_REASON_LENGTH;
pub use finish_reason::STOP as FINISH_REASON_STOP;
pub use finish_reason::TOOL_CALLS as FINISH_REASON_TOOL_CALLS;

pub use tool_call_type::CUSTOM as TOOL_CALL_TYPE_CUSTOM;
pub use tool_call_type::FUNCTION as TOOL_CALL_TYPE_FUNCTION;
