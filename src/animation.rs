// ---------------------------------------------------------------------------
// Animations
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;
use crate::{default_loop_mode, default_interpolation, default_one, Vec3, HashMap, String, Vec};

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
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}
