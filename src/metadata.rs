use crate::String;
#[cfg(not(feature = "std"))]
use crate::alloc::string::ToString;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// The `model_format` variants understood by Blockbench.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModelFormat {
    #[default]
    Free,
    #[serde(rename = "java_block")]
    JavaBlock,
    Bedrock,
    #[serde(rename = "bedrock_old")]
    BedrockOld,
    Other(String),
}

/// File-level metadata: format version, model flavour, and UV mode.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct Metadata {
    pub format_version: String,
    pub model_format: ModelFormat,
    #[serde(default)]
    pub box_uv: bool,
    #[serde(default)]
    pub backup: Option<bool>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            format_version: "5.0".to_string(),
            model_format: ModelFormat::Free,
            box_uv: false,
            backup: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

/// Default / fallback texture dimensions for UV mapping.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}
