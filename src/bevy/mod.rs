use std::{collections::HashMap, path::PathBuf};

use crate::prelude::BBModel;
use crate::{v3_x, v3_y, v3_z};
use base64::Engine;
use bevy_app::{App, Plugin, Update};
use bevy_asset::{
    Asset, AssetApp, AssetEvent, AssetId, AssetServer, Assets, Handle, RenderAssetUsages,
};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    message::MessageReader,
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Commands, Query, Res, ResMut},
};
use bevy_image::{CompressedImageFormats, Image};
use bevy_log::warn;
use bevy_mesh::{Indices, Mesh, Mesh3d, PrimitiveTopology};
use bevy_pbr::{MeshMaterial3d, StandardMaterial};
use bevy_reflect::TypePath;

// ── Plugin ────────────────────────────────────────────────────────────

/// Bevy plugin for loading and rendering `.bbmodel` (Blockbench) files.
///
/// Registers the `.bbmodel` JSON asset loader and sets up systems that
/// automatically generate meshes and materials when a model finishes
/// loading.
///
/// # Quick start
///
/// ```ignore
/// use bevy::prelude::*;
/// use bbmodel_rs::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(BBModelPlugin)
///     .add_systems(Startup, load_model)
///     .add_systems(Update, spawn_models)
///     .run();
///
/// fn load_model(asset_server: Res<AssetServer>, mut commands: Commands) {
///     let handle: Handle<BBModel> = asset_server.load("path/to/model.bbmodel");
///     commands.spawn(BBModelHandle(handle));
/// }
///
/// fn spawn_models(
///     unready: Query<(Entity, &BBModelHandle)>,
///     outputs: Res<Assets<BBModelOutput>>,
///     mut commands: Commands,
/// ) {
///     for (entity, handle) in &unready {
///         if let Some(output) = outputs.get(AssetId::from(&handle.0)) {
///             commands.entity(entity).remove::<BBModelHandle>().despawn();
///             BBModelSpawner::spawn_one(&mut commands, output);
///         }
///     }
/// }
/// ```
pub struct BBModelPlugin;

impl Plugin for BBModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<BBModel>::new(&["bbmodel"]))
            .init_asset::<BBModelOutput>()
            .init_resource::<BBModelTextureState>()
            .add_systems(
                Update,
                (
                    queue_texture_loads,
                    finalize_bbmodel_output,
                    auto_spawn_bbmodel,
                )
                    .chain(),
            );
    }
}

// ── Output asset ─────────────────────────────────────────────────────

/// Ready-to-render mesh-and-material pairs produced from a [`BBModel`].
///
/// Each entry groups all triangles that share the same texture index.
/// A `None` texture index means the face had no assigned texture
/// (it will use a plain-white fallback material).
#[derive(Asset, TypePath, Clone)]
pub struct BBModelOutput {
    /// One section per distinct texture index in the model.
    pub sections: Vec<BBModelSection>,
}

/// A single mesh-material pair for one texture index.
#[derive(Clone)]
pub struct BBModelSection {
    /// The generated mesh handle.
    pub mesh: Handle<Mesh>,
    /// The material handle (colour + optional texture).
    pub material: Handle<StandardMaterial>,
    /// Which texture index this section belongs to (`None` = untextured).
    pub texture_index: Option<u32>,
}

// ── Marker component ─────────────────────────────────────────────────

/// Marker component placed on an entity to signal "this entity is waiting
/// for its [`BBModel`] to finish processing".  When the associated
/// [`BBModelOutput`] becomes available the marker is consumed and the
/// actual mesh entities are spawned in its place.
#[derive(Component, Clone)]
pub struct BBModelHandle {
    pub model: Handle<BBModel>,
}

#[derive(Component, Clone)]
pub struct BBModelOutputHandle {
    pub output: Handle<BBModelOutput>,
}

// ── Internal state ───────────────────────────────────────────────────

/// Tracks which texture loads are in-flight for which model.
#[derive(Resource, Default)]
struct BBModelTextureState {
    /// model AssetId → list of texture paths we are loading
    pending: HashMap<AssetId<BBModel>, Vec<PathBuf>>,
    /// model AssetId → texture-index → Handle<Image>
    loaded: HashMap<AssetId<BBModel>, HashMap<u32, Handle<Image>>>,
}

// ── System 1: queue texture loads ────────────────────────────────────

/// Decode a `data:image/png;base64,...` URI into raw PNG bytes.
fn decode_data_uri(source: &str) -> Option<Vec<u8>> {
    // Strip the `data:image/...;base64,` prefix.
    let comma = source.find(',')?;
    let meta = &source[..comma];
    if !meta.contains("base64") {
        return None;
    }
    let b64 = &source[comma + 1..];
    base64::engine::general_purpose::STANDARD.decode(b64).ok()
}

/// When a `BBModel` finishes loading, start loading every texture it
/// references.  Embedded base64 textures are decoded directly into
/// `Assets<Image>`; file-referenced textures are queued with the
/// `AssetServer`.
fn queue_texture_loads(
    mut events: MessageReader<AssetEvent<BBModel>>,
    models: Res<Assets<BBModel>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<BBModelTextureState>,
) {
    for event in events.read() {
        let AssetEvent::LoadedWithDependencies { id } = event else {
            continue;
        };
        let Some(model) = models.get(*id) else {
            continue;
        };

        // Resolve the directory the .bbmodel file lives in so we can
        // build correct relative paths to textures.
        let model_dir = asset_server
            .get_path(*id)
            .and_then(|p| p.parent().map(|parent| parent.path().to_path_buf()));

        let mut texture_paths: Vec<PathBuf> = Vec::new();
        let mut index_map: HashMap<u32, Handle<Image>> = HashMap::new();

        for (idx, tex) in model.textures.iter().enumerate() {
            let tex_idx = tex
                .id
                .as_ref()
                .and_then(|id_str| id_str.parse::<u32>().ok())
                .unwrap_or(idx as u32);

            // 1) Embedded base64 texture — decode directly into an Image.
            if let Some(source) = tex.source.as_ref() {
                if let Some(png_bytes) = decode_data_uri(source) {
                    match Image::from_buffer(
                        &png_bytes,
                        bevy_image::ImageType::Format(bevy_image::ImageFormat::Png),
                        CompressedImageFormats::all(),
                        true,
                        bevy_image::ImageSampler::Default,
                        RenderAssetUsages::default(),
                    ) {
                        Ok(image) => {
                            let handle = images.add(image);
                            index_map.insert(tex_idx, handle);
                            continue;
                        }
                        Err(e) => {
                            warn!(
                                "BBModel {} texture index {}: failed to decode embedded image: {}",
                                id, tex_idx, e
                            );
                        }
                    }
                } else {
                    warn!(
                        "BBModel {} texture index {}: malformed data URI",
                        id, tex_idx
                    );
                }
            }

            // 2) File-referenced texture — load via the AssetServer.
            let rel = tex.relative_path.as_ref().or(tex.path.as_ref());
            let Some(rel) = rel else {
                warn!(
                    "BBModel {} texture index {} has no source or path; skipping",
                    id, tex_idx
                );
                continue;
            };

            let full_path = if let Some(ref dir) = model_dir {
                dir.join(rel)
            } else {
                rel.to_path_buf()
            };

            texture_paths.push(full_path.clone());
            index_map.insert(tex_idx, asset_server.load(full_path));
        }

        state.pending.insert(*id, texture_paths);
        state.loaded.insert(*id, index_map);
    }
}

// ── System 2: produce BBModelOutput ──────────────────────────────────

/// Once all textures for a model have loaded, triangulate the model
/// into one mesh per texture index and emit a [`BBModelOutput`].
fn finalize_bbmodel_output(
    mut commands: Commands,
    models: Res<Assets<BBModel>>,
    images: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<BBModelTextureState>,
    mut outputs: ResMut<Assets<BBModelOutput>>,
) {
    // Snapshot the pending keys so we don't hold `state` across the
    // mutable borrows below.
    let ready: Vec<AssetId<BBModel>> = state
        .pending
        .iter()
        .filter_map(|(id, paths)| {
            // Check the handles we stored earlier.
            let handles = state.loaded.get(id);
            handles.and_then(|hmap| {
                let all_ready = hmap.values().all(|h| images.get(h).is_some());
                if all_ready { Some(*id) } else { None }
            })
        })
        .collect();

    for id in &ready {
        let Some(model) = models.get(*id) else {
            continue;
        };
        let Some(loaded_textures) = state.loaded.remove(id) else {
            continue;
        };
        state.pending.remove(id);

        // ── Group triangles by texture index ─────────────────────
        let mut sections: HashMap<
            Option<u32>,
            (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<[f32; 3]>, Vec<u32>),
        > = HashMap::new();
        // (positions, uvs, normals, indices)

        let mut next_index: u32 = 0;

        // Build a per-texture-index resolution lookup so we can normalize UVs.
        // Blockbench stores UVs in pixel space with V pointing *down*; Bevy
        // expects normalized [0,1] UVs with V pointing *up*.
        //
        // Prefer the *actual* dimensions of the loaded Image (most reliable),
        // then the per-texture `uv_width`/`uv_height`, then `width`/`height`,
        // then the model `resolution`, then a sane default.
        let default_res = model
            .resolution
            .as_ref()
            .map(|r| (r.width as f32, r.height as f32))
            .unwrap_or((16.0, 16.0));

        let tex_resolution: HashMap<u32, (f32, f32)> = model
            .textures
            .iter()
            .enumerate()
            .map(|(idx, tex)| {
                let tex_idx = tex
                    .id
                    .as_ref()
                    .and_then(|id_str| id_str.parse::<u32>().ok())
                    .unwrap_or(idx as u32);

                // Try the loaded image first.
                let from_image = loaded_textures.get(&tex_idx).and_then(|h| {
                    images
                        .get(h)
                        .map(|img| (img.width() as f32, img.height() as f32))
                });

                let (w, h) = from_image
                    .or_else(|| {
                        tex.uv_width
                            .zip(tex.uv_height)
                            .map(|(w, h)| (w as f32, h as f32))
                    })
                    .or_else(|| tex.width.zip(tex.height).map(|(w, h)| (w as f32, h as f32)))
                    .unwrap_or(default_res);

                (tex_idx, (w, h))
            })
            .collect();

        for element in &model.elements {
            for tri in element.triangulate() {
                let tex_idx = tri.texture;
                let entry = sections.entry(tex_idx).or_default();
                // Resolve the UV resolution for this triangle's texture.
                let (uw, uh) = tex_idx
                    .and_then(|idx| tex_resolution.get(&idx).copied())
                    .unwrap_or(default_res);
                // Guard against zero-sized textures to avoid NaNs.
                let uw = if uw > 0.0 { uw } else { default_res.0 };
                let uh = if uh > 0.0 { uh } else { default_res.1 };
                // Push 3 vertices.
                for vi in 0..3 {
                    entry.0.push(to_f32x3(&tri.positions[vi]));
                    // Normalize UV: divide by texture size and flip V.
                    let u = to_f32x2(&tri.uvs[vi])[0] / uw;
                    let v = to_f32x2(&tri.uvs[vi])[1] / uh;
                    entry.1.push([u, 1.0 - v]);
                    entry.2.push(to_f32x3(&tri.normals[vi]));
                }
                // Push 3 indices.
                entry.3.push(next_index);
                entry.3.push(next_index + 1);
                entry.3.push(next_index + 2);
                next_index += 3;
            }
        }

        // ── Build mesh + material per section ───────────────────
        let mut output_sections: Vec<BBModelSection> = Vec::new();

        for (tex_idx, (positions, uvs, normals, indices)) in &sections {
            if positions.is_empty() {
                continue;
            }

            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone())
            .with_inserted_indices(Indices::U32(indices.clone()));

            let mesh_handle = meshes.add(mesh);

            let material = if let Some(handle) = tex_idx
                .and_then(|idx| loaded_textures.get(&idx))
                .filter(|h| images.get(*h).is_some())
            {
                StandardMaterial {
                    base_color_texture: Some(handle.clone()),
                    ..Default::default()
                }
            } else {
                // Untextured fallback — a visible magenta so missing
                // textures are obvious rather than invisible.
                StandardMaterial {
                    base_color: bevy_color::Color::srgb(1.0, 0.0, 1.0),
                    ..Default::default()
                }
            };

            let material_handle = materials.add(material);

            output_sections.push(BBModelSection {
                mesh: mesh_handle,
                material: material_handle,
                texture_index: *tex_idx,
            });
        }

        let output = outputs.add(BBModelOutput {
            sections: output_sections,
        });
        commands.spawn(BBModelOutputHandle { output });
    }
}

// ── System 3: auto-spawn entities ────────────────────────────────────

/// Optional convenience system: any entity carrying a [`BBModelHandle`]
/// is replaced with the mesh/material entities as soon as the
/// [`BBModelOutput`] is available.
///
/// If you prefer to control spawning yourself, remove this system and
/// query [`BBModelOutput`] directly.
fn auto_spawn_bbmodel(
    markers: Query<(Entity, &BBModelOutputHandle)>,
    outputs: Res<Assets<BBModelOutput>>,
    mut commands: Commands,
) {
    for (entity, handle) in &markers {
        if let Some(output) = outputs.get(&handle.output) {
            // Despawn the placeholder.
            commands.entity(entity).despawn();
            BBModelSpawner::spawn_one(&mut commands, output);
        }
    }
}

// ── Public spawner API ───────────────────────────────────────────────

/// Helper for spawning a processed [`BBModelOutput`] into the world.
pub struct BBModelSpawner;

impl BBModelSpawner {
    /// Spawn every mesh section as a child of a new root entity.
    ///
    /// Returns the root [`Entity`].
    pub fn spawn_one(commands: &mut Commands, output: &BBModelOutput) -> Entity {
        let root = commands
            .spawn(bevy_transform::components::Transform::default())
            .id();

        for section in &output.sections {
            let child = commands
                .spawn((
                    Mesh3d(section.mesh.clone()),
                    MeshMaterial3d(section.material.clone()),
                    bevy_transform::components::Transform::default(),
                ))
                .id();
            commands.entity(root).add_child(child);
        }

        root
    }

    /// Spawn all sections as children of an **existing** entity.
    pub fn spawn_as_children(commands: &mut Commands, parent: Entity, output: &BBModelOutput) {
        for section in &output.sections {
            let child = commands
                .spawn((
                    Mesh3d(section.mesh.clone()),
                    MeshMaterial3d(section.material.clone()),
                    bevy_transform::components::Transform::default(),
                ))
                .id();
            commands.entity(parent).add_child(child);
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Convert a [`crate::Vec3`] (which may be `glam::Vec3` or `[f32;3]`)
/// into a plain `[f32; 3]` for Bevy mesh attributes.
#[cfg(feature = "glam")]
fn to_f32x3(v: &crate::Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

#[cfg(not(feature = "glam"))]
fn to_f32x3(v: &crate::Vec3) -> [f32; 3] {
    [v[0], v[1], v[2]]
}

/// Convert a [`crate::Vec2`] (which may be `glam::Vec2` or `[f32;2]`)
/// into a plain `[f32; 2]` for Bevy mesh UV attributes.
#[cfg(feature = "glam")]
fn to_f32x2(v: &crate::Vec2) -> [f32; 2] {
    [v.x, v.y]
}

#[cfg(not(feature = "glam"))]
fn to_f32x2(v: &crate::Vec2) -> [f32; 2] {
    [v[0], v[1]]
}
