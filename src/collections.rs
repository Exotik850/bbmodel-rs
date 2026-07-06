// ---------------------------------------------------------------------------
// Groups (v5.0 style - defined outside the outliner)
// ---------------------------------------------------------------------------
use crate::{String, Vec, Vec3, default_true};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

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
/// separate `groups` array - this struct only carries hierarchy information.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct OutlinerGroup {
    pub uuid: Uuid,
    pub name: Option<String>,
    #[serde(default)]
    pub origin: Option<Vec3>,
    #[serde(default)]
    pub rotation: Option<Vec3>,
    #[serde(rename = "isOpen", default)]
    pub is_open: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<OutlinerNode>,
}

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
