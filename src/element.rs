// ---------------------------------------------------------------------------
// Elements (geometry primitives)
// ---------------------------------------------------------------------------

use crate::{UvRect, Vec2, Vec3, HashMap, default_true, String, Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

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
