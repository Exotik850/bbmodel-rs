#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

pub mod prelude {
    pub use crate::animation::*;
    pub use crate::collections::*;
    pub use crate::element::*;
    pub use crate::metadata::*;
    pub use crate::texture::*;
    pub use crate::{BBModel, Display, DisplaySlot, Tri, UvRect, Vec2, Vec3};
}
pub use prelude::*;
pub mod animation;
pub mod collections;
pub mod element;
pub mod metadata;
pub mod texture;

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

cfg_select! {
    feature = "std" => {
        pub type HashMap<K, V> = std::collections::HashMap<K, V>;
        pub type PathBuf = std::path::PathBuf;
        pub type String = std::string::String;
        pub type Vec<T> = std::vec::Vec<T>;
    }
    not(feature = "std") => {
        extern crate alloc;
        pub type HashMap<K, V> = alloc::collections::BTreeMap<K, V>;
        pub type PathBuf = alloc::string::String;
        pub type String = alloc::string::String;
        pub type Vec<T> = alloc::vec::Vec<T>;
    }
}

// ---------------------------------------------------------------------------
// Vec3 / Vec2 accessor helpers (abstract over glam vs arrays)
// ---------------------------------------------------------------------------

#[cfg(not(feature = "glam"))]
pub(crate) fn v3(x: f32, y: f32, z: f32) -> Vec3 { [x, y, z] }
#[cfg(feature = "glam")]
pub(crate) fn v3(x: f32, y: f32, z: f32) -> Vec3 { glam::Vec3::new(x, y, z) }

#[cfg(not(feature = "glam"))]
pub(crate) fn v2(u: f32, v: f32) -> Vec2 { [u, v] }
#[cfg(feature = "glam")]
pub(crate) fn v2(u: f32, v: f32) -> Vec2 { glam::Vec2::new(u, v) }

#[cfg(not(feature = "glam"))]
pub(crate) fn v3_x(v: &Vec3) -> f32 { v[0] }
#[cfg(feature = "glam")]
pub(crate) fn v3_x(v: &Vec3) -> f32 { v.x }

#[cfg(not(feature = "glam"))]
pub(crate) fn v3_y(v: &Vec3) -> f32 { v[1] }
#[cfg(feature = "glam")]
pub(crate) fn v3_y(v: &Vec3) -> f32 { v.y }

#[cfg(not(feature = "glam"))]
pub(crate) fn v3_z(v: &Vec3) -> f32 { v[2] }
#[cfg(feature = "glam")]
pub(crate) fn v3_z(v: &Vec3) -> f32 { v.z }

#[allow(dead_code)]
#[cfg(not(feature = "glam"))]
pub(crate) fn v2_x(v: &Vec2) -> f32 { v[0] }
#[allow(dead_code)]
#[cfg(feature = "glam")]
pub(crate) fn v2_x(v: &Vec2) -> f32 { v.x }

#[allow(dead_code)]
#[cfg(not(feature = "glam"))]
pub(crate) fn v2_y(v: &Vec2) -> f32 { v[1] }
#[allow(dead_code)]
#[cfg(feature = "glam")]
pub(crate) fn v2_y(v: &Vec2) -> f32 { v.y }

#[cfg(not(feature = "glam"))]
pub(crate) fn v4_x(v: &UvRect) -> f32 { v[0] }
#[cfg(feature = "glam")]
pub(crate) fn v4_x(v: &UvRect) -> f32 { v.x }

#[cfg(not(feature = "glam"))]
pub(crate) fn v4_y(v: &UvRect) -> f32 { v[1] }
#[cfg(feature = "glam")]
pub(crate) fn v4_y(v: &UvRect) -> f32 { v.y }

#[cfg(not(feature = "glam"))]
pub(crate) fn v4_z(v: &UvRect) -> f32 { v[2] }
#[cfg(feature = "glam")]
pub(crate) fn v4_z(v: &UvRect) -> f32 { v.z }

#[cfg(not(feature = "glam"))]
pub(crate) fn v4_w(v: &UvRect) -> f32 { v[3] }
#[cfg(feature = "glam")]
pub(crate) fn v4_w(v: &UvRect) -> f32 { v.w }

#[cfg(feature = "std")]
pub(crate) fn sqrt(x: f32) -> f32 { x.sqrt() }
#[cfg(not(feature = "std"))]
pub(crate) fn sqrt(x: f32) -> f32 {
    // IEEE 754 fast inverse sqrt trick for no_std
    if x <= 0.0 { return 0.0; }
    let i = f32::to_bits(x);
    let i = 0x1fbd1df5 + (i >> 1);
    let y = f32::from_bits(i);
    let y = 0.5 * (y + x / y);
    0.5 * (y + x / y)
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
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[skip_serializing_none]
pub struct BBModel {
    pub meta: Metadata,
    #[serde(default)]
    pub name: Option<String>,
    pub resolution: Option<Resolution>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<Element>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<Group>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outliner: Vec<OutlinerNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub textures: Vec<Texture>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub animations: Vec<Animation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub animation_controllers: Vec<AnimationController>,
    #[serde(default)]
    pub display: Option<Display>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collections: Vec<Collection>,
}

impl BBModel {
    pub fn new() -> Self {
        BBModel::default()
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> &mut Self {
        self.elements.push(Element::Mesh(mesh));
        self
    }

    /// Iterate all render-ready triangles from every element.
    ///
    /// This is a zero-allocation convenience over calling
    /// [`Element::triangulate`] on each element individually.
    pub fn triangles(&self) -> impl Iterator<Item = Tri> + '_ {
        self.elements.iter().flat_map(|el| el.triangulate())
    }

    /// Iterate only the [`Cube`] elements.
    pub fn cubes(&self) -> impl Iterator<Item = &Cube> + '_ {
        self.elements.iter().filter_map(|el| match el {
            Element::Cube(c) => Some(c),
            _ => None,
        })
    }

    /// Iterate only the [`Mesh`] elements.
    pub fn meshes(&self) -> impl Iterator<Item = &Mesh> + '_ {
        self.elements.iter().filter_map(|el| match el {
            Element::Mesh(m) => Some(m),
            _ => None,
        })
    }

    /// Iterate elements that are both exported and visible.
    pub fn visible_elements(&self) -> impl Iterator<Item = &Element> + '_ {
        self.elements.iter().filter(|el| match el {
            Element::Cube(c) => c.export && c.visibility,
            Element::Mesh(m) => m.export && m.visibility,
        })
    }

    /// Find an element by its UUID.
    pub fn find_element(&self, uuid: &Uuid) -> Option<&Element> {
        self.elements.iter().find(|el| el.uuid() == uuid)
    }

    /// Find a group by its UUID.
    pub fn find_group(&self, uuid: &Uuid) -> Option<&Group> {
        self.groups.iter().find(|g| &g.uuid == uuid)
    }

    /// Find a texture by its UUID.
    pub fn find_texture(&self, uuid: &Uuid) -> Option<&Texture> {
        self.textures.iter().find(|t| t.uuid.as_ref() == Some(uuid))
    }

    /// Find a collection by its UUID.
    pub fn find_collection(&self, uuid: &Uuid) -> Option<&Collection> {
        self.collections.iter().find(|c| &c.uuid == uuid)
    }
}

// ---------------------------------------------------------------------------
// Render-ready geometry
// ---------------------------------------------------------------------------

/// A single render-ready triangle produced by triangulating a model element.
///
/// Each triangle carries per-vertex positions, UV coordinates, and normals
/// along with an optional texture index referencing the model's [`Texture`]
/// array.
#[derive(Debug, Clone)]
pub struct Tri {
    /// Three vertex positions in model space.
    pub positions: [Vec3; 3],
    /// Three per-vertex UV coordinates.
    pub uvs: [Vec2; 3],
    /// Three per-vertex normals (unit vectors).
    pub normals: [Vec3; 3],
    /// Index into the model's `textures` array, if any.
    pub texture: Option<u32>,
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

#[cfg(all(test, feature = "std"))]
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
