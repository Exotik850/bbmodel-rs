// ---------------------------------------------------------------------------
// Textures
// ---------------------------------------------------------------------------

use crate::{PathBuf, default_true, String};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

/// Render mode for a texture.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    Default,
    Normal,
    Emissive,
    Layered,
}

/// Which sides a texture is rendered on.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderSides {
    Auto,
    Front,
    Double,
}

/// A texture definition.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[skip_serializing_none]
#[serde(default)]
pub struct Texture {
    /// Unique identifier for this texture (UUIDv4).
    pub uuid: Option<Uuid>,
    /// Optional human-readable name (e.g. `"steve_skin"`).
    pub name: Option<String>,
    /// Texture index (e.g. `"0"`, `"1"`).
    pub id: Option<String>,
    /// Absolute filesystem path (may be stale).
    pub path: Option<PathBuf>,
    /// Path relative to the directory containing the `.bbmodel` file.
    pub relative_path: Option<PathBuf>,
    /// Image pixel width.
    pub width: Option<u32>,
    /// Image pixel height.
    pub height: Option<u32>,
    /// UV coordinate space width (per-texture, v4.9+).
    pub uv_width: Option<u32>,
    /// UV coordinate space height (per-texture, v4.9+).
    pub uv_height: Option<u32>,
    /// Whether the texture is a particle texture.
    #[serde(default)]
    pub particle: bool,
    /// Rendering mode for this texture (default, normal, emissive, layered).
    pub render_mode: Option<RenderMode>,
    /// Which sides of a cube the texture is rendered on (auto, front, double).
    pub render_sides: Option<RenderSides>,
    /// Whether the texture data is embedded in this file.
    pub internal: bool,
    /// Base64-encoded image data (`data:image/png;base64,…`).
    pub source: Option<String>,
    pub mode: Option<String>,
    pub saved: bool,
    #[serde(default = "default_true")]
    pub visible: bool,
    pub folder: Option<String>,
    pub namespace: Option<String>,
}
