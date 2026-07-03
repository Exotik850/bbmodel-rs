#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub mod prelude {
    pub use crate::animation::*;
    pub use crate::collections::*;
    pub use crate::element::*;
    pub use crate::metadata::*;
    pub use crate::texture::*;
    pub use crate::{BBModel, Display, DisplaySlot, UvRect, Vec2, Vec3};
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
