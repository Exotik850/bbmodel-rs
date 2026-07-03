#![cfg_attr(not(feature = "std"), no_std)]


use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

cfg_select! {
  feature = "glam" => {
    /// A point or vector in 3D space: `[x, y, z]`.
    pub type Vec3 = glam::Vec3;
    /// A 2D coordinate or vector: `[u, v]`.
    pub type Vec2 = glam::Vec2;
    /// A UV rectangle: `[u_min, v_min, u_max, v_max]`.
    pub type UvRect = glam::Vec4;
  }
  not(feature = "glam") => {
    /// A point or vector in 3D space: `[x, y, z]`.
    pub type Vec3 = [f32; 3];
    /// A 2D coordinate or vector: `[u, v]`.
    pub type Vec2 = [f32; 2];
    /// A UV rectangle: `[u_min, v_min, u_max, v_max]`.
    pub type UvRect = [f32; 4];
  }
}

// ---------------------------------------------------------------------------
// Serde default helpers
// ---------------------------------------------------------------------------

fn default_true() -> bool {
    true
}

fn default_loop_mode() -> LoopMode {
    LoopMode::Loop
}

fn default_one() -> f32 {
    1.0
}

fn default_interpolation() -> Interpolation {
    Interpolation::Linear
}

// ---------------------------------------------------------------------------
// Root document
// ---------------------------------------------------------------------------

/// Top-level `.bbmodel` file representation (format version 5.0).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct BBModel {
    pub meta: Metadata,
    #[serde(default)]
    pub name: Option<String>,
    pub resolution: Resolution,
    #[serde(default)]
    pub elements: Vec<Element>,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub outliner: Vec<OutlinerNode>,
    #[serde(default)]
    pub textures: Vec<Texture>,
    #[serde(default)]
    pub animations: Vec<Animation>,
    #[serde(default)]
    pub animation_controllers: Vec<AnimationController>,
    #[serde(default)]
    pub display: Option<Display>,
    #[serde(default)]
    pub collections: Vec<Collection>,
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

/// The `model_format` variants understood by Blockbench.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFormat {
    Free,
    #[serde(rename = "java_block")]
    JavaBlock,
    Bedrock,
    #[serde(rename = "bedrock_old")]
    BedrockOld,
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

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

/// Default / fallback texture dimensions for UV mapping.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

// ---------------------------------------------------------------------------
// Elements (geometry primitives)
// ---------------------------------------------------------------------------

/// A geometric primitive — either a cube or a freeform mesh.
///
/// **Deserialization**: uses `#[serde(untagged)]` because cubes have no
/// `"type"` field in the JSON (they are the implicit default).  Meshes are
/// distinguished by their `"type": "mesh"` field and structurally different
/// shape (`vertices`/`faces` maps vs. `from`/`to` arrays).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Element {
    /// Tried first — cubes have no `type` tag.
    Cube(Cube),
    /// Falls through when `Cube` fails (no `from`/`to` fields).
    Mesh(Mesh),
}

// ---------------------------------------------------------------------------
// Cube
// ---------------------------------------------------------------------------

/// An axis-aligned box primitive with six optionally-textured faces.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct Cube {
    #[serde(default)]
    pub name: Option<String>,
    pub uuid: Uuid,
    /// Near corner `[x, y, z]`.
    pub from: Vec3,
    /// Far corner `[x, y, z]`.
    pub to: Vec3,
    /// Pivot point for rotation.
    #[serde(default)]
    pub origin: Vec3,
    /// Euler rotation in degrees `[x, y, z]` (applied X→Y→Z around `origin`).
    #[serde(default)]
    pub rotation: Vec3,
    /// Colour palette index (0–7 for Minecraft).
    #[serde(default)]
    pub color: u32,
    #[serde(default = "default_true")]
    pub export: bool,
    #[serde(default = "default_true")]
    pub visibility: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub box_uv: bool,
    #[serde(default)]
    pub rescale: bool,
    #[serde(default)]
    pub autouv: u32,
    #[serde(default)]
    pub mirror_uv: bool,
    #[serde(default)]
    pub faces: CubeFaces,
}

/// Named faces of a cube (north, east, south, west, up, down).
///
/// Any face set to `None` is absent / culled.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[skip_serializing_none]
pub struct CubeFaces {
    #[serde(default)]
    pub north: Option<CubeFace>,
    #[serde(default)]
    pub east: Option<CubeFace>,
    #[serde(default)]
    pub south: Option<CubeFace>,
    #[serde(default)]
    pub west: Option<CubeFace>,
    #[serde(default)]
    pub up: Option<CubeFace>,
    #[serde(default)]
    pub down: Option<CubeFace>,
}

/// A single cube face with UV coordinates and an optional texture index.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CubeFace {
    /// UV rectangle `[u_min, v_min, u_max, v_max]` in texture space.
    pub uv: UvRect,
    #[serde(default)]
    pub texture: Option<u32>,
}

// ---------------------------------------------------------------------------
// Mesh
// ---------------------------------------------------------------------------

/// A freeform mesh with arbitrary vertices and faces.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct Mesh {
    #[serde(default)]
    pub name: Option<String>,
    pub uuid: Uuid,
    #[serde(default)]
    pub origin: Vec3,
    #[serde(default)]
    pub rotation: Vec3,
    #[serde(default)]
    pub color: u32,
    #[serde(default = "default_true")]
    pub export: bool,
    #[serde(default = "default_true")]
    pub visibility: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub render_order: Option<String>,
    #[serde(default)]
    pub allow_mirror_modeling: bool,
    #[serde(default)]
    pub mirror_uv: bool,
    /// Vertex positions keyed by vertex ID.
    #[serde(default)]
    pub vertices: HashMap<String, Vec3>,
    /// Mesh faces keyed by face ID.
    #[serde(default)]
    pub faces: HashMap<String, MeshFace>,
}

/// A single mesh face (quad or triangle) with per-vertex UVs.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct MeshFace {
    /// Per-vertex UV coordinates: vertex ID → `[u, v]`.
    #[serde(default)]
    pub uv: HashMap<String, Vec2>,
    /// Ordered vertex IDs (CCW winding for outward-facing).
    pub vertices: Vec<String>,
    #[serde(default)]
    pub texture: Option<u32>,
}

// ---------------------------------------------------------------------------
// Groups (v5.0 style — defined outside the outliner)
// ---------------------------------------------------------------------------

/// A transform group that lives in the `groups` array.
///
/// Children are determined by the `outliner` entries that reference this
/// group's UUID.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct Group {
    #[serde(default)]
    pub name: Option<String>,
    pub uuid: Uuid,
    #[serde(default)]
    pub origin: Vec3,
    #[serde(default)]
    pub rotation: Vec3,
    #[serde(default)]
    pub color: u32,
    #[serde(default = "default_true")]
    pub export: bool,
    #[serde(default)]
    pub mirror_uv: bool,
    #[serde(rename = "isOpen", default)]
    pub is_open: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default = "default_true")]
    pub visibility: bool,
    #[serde(default)]
    pub autouv: u32,
    /// Arbitrary NBT data (JSON-encoded string, Minecraft-specific).
    #[serde(default)]
    pub nbt: Option<String>,
}

// ---------------------------------------------------------------------------
// Outliner / scene hierarchy
// ---------------------------------------------------------------------------

/// A node in the outliner tree.
///
/// Can be a leaf element (UUID string) or a group wrapper with children.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OutlinerNode {
    /// A leaf geometry element referenced by its UUID.
    Element(Uuid),
    /// A group node containing child nodes.
    Group(OutlinerGroup),
}

/// A group wrapper used exclusively inside the outliner tree.
///
/// In v5.0+ the "heavy" properties (name, origin, rotation, …) live in the
/// separate `groups` array — this struct only carries hierarchy information.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct OutlinerGroup {
    pub uuid: Uuid,
    pub name: String,
    #[serde(default)]
    pub origin: Option<Vec3>,
    #[serde(default)]
    pub rotation: Option<Vec3>,
    #[serde(rename = "isOpen", default)]
    pub is_open: bool,
    #[serde(default)]
    pub children: Vec<OutlinerNode>,
}

// ---------------------------------------------------------------------------
// Textures
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Animations
// ---------------------------------------------------------------------------

/// Loop behaviour of an animation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopMode {
    Once,
    Loop,
    Hold,
}

/// A named animation containing animators keyed by bone UUID.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct Animation {
    pub uuid: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "loop", default = "default_loop_mode")]
    pub r#loop: LoopMode,
    #[serde(rename = "override", default)]
    pub r#override: bool,
    #[serde(default = "default_one")]
    pub length: f32,
    #[serde(default)]
    pub snapping: u32,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub anim_time_update: Option<String>,
    #[serde(default)]
    pub blend_weight: Option<String>,
    #[serde(default)]
    pub start_delay: Option<String>,
    #[serde(default)]
    pub loop_delay: Option<String>,
    /// Animators keyed by the animated node/element UUID.
    #[serde(default)]
    pub animators: HashMap<String, Animator>,
}

/// The kind of node being animated.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimatorType {
    Bone,
}

/// Per-bone animation data: a named animator with keyframes.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct Animator {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "type", default)]
    pub r#type: Option<AnimatorType>,
    #[serde(default)]
    pub keyframes: Vec<Keyframe>,
}

/// Which transform component a keyframe targets.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyframeChannel {
    Position,
    Rotation,
    Scale,
}

/// Interpolation method between keyframes.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Interpolation {
    Linear,
    Step,
    #[serde(rename = "catmullrom")]
    CatmullRom,
    Bezier,
}

/// A single keyframe at a point in time.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct Keyframe {
    pub channel: KeyframeChannel,
    pub uuid: Uuid,
    /// Time in seconds.
    pub time: f32,
    #[serde(default)]
    pub color: i32,
    #[serde(default = "default_interpolation")]
    pub interpolation: Interpolation,
    /// One value = continuous; two values = discontinuous (instant jump).
    pub data_points: Vec<Vec3>,
    // --- Bezier handle data ---
    #[serde(default)]
    pub bezier_linked: Option<bool>,
    #[serde(default)]
    pub bezier_left_time: Option<Vec3>,
    #[serde(default)]
    pub bezier_left_value: Option<Vec3>,
    #[serde(default)]
    pub bezier_right_time: Option<Vec3>,
    #[serde(default)]
    pub bezier_right_value: Option<Vec3>,
}

// ---------------------------------------------------------------------------
// Animation controllers (state machines)
// ---------------------------------------------------------------------------

/// An animation controller / state machine.
///
/// The exact structure varies; this representation captures common fields
/// while preserving any extra data.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimationController {
    #[serde(default)]
    pub uuid: Option<Uuid>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Display settings (Minecraft item transforms)
// ---------------------------------------------------------------------------

/// A per-slot display transform (rotation, translation, scale).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DisplaySlot {
    #[serde(default)]
    pub rotation: Vec3,
    #[serde(default)]
    pub translation: Vec3,
    #[serde(default)]
    pub scale: Vec3,
}

/// Item-display transformations keyed by slot name.
///
/// Common slots: `thirdperson_righthand`, `thirdperson_lefthand`,
/// `firstperson_righthand`, `firstperson_lefthand`, `gui`, `ground`,
/// `fixed`, `head`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Display(pub HashMap<String, DisplaySlot>);

// ---------------------------------------------------------------------------
// Collections (v5.0+)
// ---------------------------------------------------------------------------

/// A named collection of element UUIDs, typically for LOD or export grouping.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct Collection {
    pub uuid: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub children: Vec<Uuid>,
    #[serde(default)]
    pub export_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::BBModel;

    #[test]
    fn test_deserialize_bbmodel() {
        let file =
            std::fs::read_to_string("./tests/model_pizza.bbmodel").expect("Unable to read file");
        let model: BBModel = serde_json::from_str(&file).expect("Failed to deserialize BBModel");
        assert_eq!(model.name.as_deref(), Some("model_pizza"));
    }
}
