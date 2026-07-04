// ---------------------------------------------------------------------------
// Elements (geometry primitives)
// ---------------------------------------------------------------------------

use crate::{HashMap, String, Tri, UvRect, Vec, Vec2, Vec3, default_true, sqrt, v2, v3, v3_x, v3_y, v3_z, v4_x, v4_y, v4_z, v4_w};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

/// A geometric primitive - either a cube or a freeform mesh.
///
/// **Deserialization**: uses `#[serde(untagged)]` because cubes have no
/// `"type"` field in the JSON (they are the implicit default).  Meshes are
/// distinguished by their `"type": "mesh"` field and structurally different
/// shape (`vertices`/`faces` maps vs. `from`/`to` arrays).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Element {
    /// Tried first - cubes have no `type` tag.
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
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
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
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
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
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub vertices: HashMap<String, Vec3>,
    /// Mesh faces keyed by face ID.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub faces: HashMap<String, MeshFace>,
}

/// A single mesh face (quad or triangle) with per-vertex UVs.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[skip_serializing_none]
pub struct MeshFace {
    /// Per-vertex UV coordinates: vertex ID → `[u, v]`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub uv: HashMap<String, Vec2>,
    /// Ordered vertex IDs (CCW winding for outward-facing).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vertices: Vec<String>,
    #[serde(default)]
    pub texture: Option<u32>,
}

// ---------------------------------------------------------------------------
// Element triangulation helpers
// ---------------------------------------------------------------------------

impl Element {
    /// Returns a shared reference to the element's UUID.
    pub fn uuid(&self) -> &Uuid {
        match self {
            Element::Cube(c) => &c.uuid,
            Element::Mesh(m) => &m.uuid,
        }
    }

    /// Triangulate this element into render-ready triangles.
    pub fn triangulate(&self) -> TriIter<'_> {
        match self {
            Element::Cube(c) => TriIter::Cube(c.iter_tris()),
            Element::Mesh(m) => TriIter::Mesh(m.iter_tris()),
        }
    }
}

/// An iterator over [`Tri`] values produced by an [`Element`].
///
/// This avoids allocating a `Vec` - triangles are yielded lazily as
/// the iterator is consumed.
pub enum TriIter<'a> {
    Cube(CubeTriIter<'a>),
    Mesh(MeshTriIter<'a>),
}

impl<'a> Iterator for TriIter<'a> {
    type Item = Tri;

    fn next(&mut self) -> Option<Tri> {
        match self {
            TriIter::Cube(inner) => inner.next(),
            TriIter::Mesh(inner) => inner.next(),
        }
    }
}

// ---------------------------------------------------------------------------
// Cube triangulation
// ---------------------------------------------------------------------------

/// The eight corner vertices of a unit cube, in CCW order per face.
/// Faces: north(-Z), east(+X), south(+Z), west(-X), up(+Y), down(-Y)
///
/// Each face = quad [(a,b,c,d)] → two triangles (a,b,c) and (a,c,d).
struct FaceDesc {
    quad: [usize; 4],
    normal: [f32; 3],
}

const CUBE_CORNERS: [[f32; 3]; 8] = [
    [0.0, 0.0, 0.0], // 0: (-,-,-)
    [1.0, 0.0, 0.0], // 1: (+,-,-)
    [1.0, 1.0, 0.0], // 2: (+,+,-)
    [0.0, 1.0, 0.0], // 3: (-,+,-)
    [0.0, 0.0, 1.0], // 4: (-,-,+)
    [1.0, 0.0, 1.0], // 5: (+,-,+)
    [1.0, 1.0, 1.0], // 6: (+,+,+)
    [0.0, 1.0, 1.0], // 7: (-,+,+)
];

const CUBE_FACES: [FaceDesc; 6] = [
    // north (-Z)
    FaceDesc { quad: [3, 2, 1, 0], normal: [0.0, 0.0, -1.0] },
    // east (+X)
    FaceDesc { quad: [2, 6, 5, 1], normal: [1.0, 0.0, 0.0] },
    // south (+Z)
    FaceDesc { quad: [7, 6, 2, 3], normal: [0.0, 0.0, 1.0] },
    // west (-X)
    FaceDesc { quad: [0, 4, 7, 3], normal: [-1.0, 0.0, 0.0] },
    // up (+Y)
    FaceDesc { quad: [3, 7, 6, 2], normal: [0.0, 1.0, 0.0] },
    // down (-Y)
    FaceDesc { quad: [0, 1, 5, 4], normal: [0.0, -1.0, 0.0] },
];

impl Cube {
    /// Lazy iterator over the triangles of this cube.
    ///
    /// Yields up to 12 triangles (2 per visible face × 6 faces).  Faces
    /// whose corresponding [`CubeFace`] is `None` are skipped.
    pub fn iter_tris(&self) -> CubeTriIter<'_> {
        CubeTriIter {
            cube: self,
            face_idx: 0,
            tri_idx: 0,
        }
    }

    fn tri_for_face(&self, face_idx: usize, tri_idx: usize) -> Option<Tri> {
        let face_opt: Option<&CubeFace> = match face_idx {
            0 => self.faces.north.as_ref(),
            1 => self.faces.east.as_ref(),
            2 => self.faces.south.as_ref(),
            3 => self.faces.west.as_ref(),
            4 => self.faces.up.as_ref(),
            5 => self.faces.down.as_ref(),
            _ => return None,
        };
        let face = face_opt?;
        let desc = &CUBE_FACES[face_idx];
        let texture = face.texture;

        let from_x = v3_x(&self.from);
        let from_y = v3_y(&self.from);
        let from_z = v3_z(&self.from);
        let to_x = v3_x(&self.to);
        let to_y = v3_y(&self.to);
        let to_z = v3_z(&self.to);

        let u_min = v4_x(&face.uv);
        let v_min = v4_y(&face.uv);
        let u_max = v4_z(&face.uv);
        let v_max = v4_w(&face.uv);

        let corners: [Vec3; 4] = desc.quad.map(|ci| {
            let c = CUBE_CORNERS[ci];
            v3(
                from_x + c[0] * (to_x - from_x),
                from_y + c[1] * (to_y - from_y),
                from_z + c[2] * (to_z - from_z),
            )
        });

        let uv_corners: [Vec2; 4] = [
            v2(u_min, v_min),
            v2(u_max, v_min),
            v2(u_max, v_max),
            v2(u_min, v_max),
        ];

        let normal = v3(desc.normal[0], desc.normal[1], desc.normal[2]);

        if tri_idx == 0 {
            Some(Tri {
                positions: [corners[0], corners[1], corners[2]],
                uvs: [uv_corners[0], uv_corners[1], uv_corners[2]],
                normals: [normal, normal, normal],
                texture,
            })
        } else {
            Some(Tri {
                positions: [corners[0], corners[2], corners[3]],
                uvs: [uv_corners[0], uv_corners[2], uv_corners[3]],
                normals: [normal, normal, normal],
                texture,
            })
        }
    }
}

/// Iterator over triangles generated from a [`Cube`]'s faces.
pub struct CubeTriIter<'a> {
    cube: &'a Cube,
    face_idx: usize,
    tri_idx: usize,
}

impl<'a> Iterator for CubeTriIter<'a> {
    type Item = Tri;

    fn next(&mut self) -> Option<Tri> {
        while self.face_idx < 6 {
            let tri = self.cube.tri_for_face(self.face_idx, self.tri_idx);
            self.tri_idx += 1;
            if self.tri_idx >= 2 {
                self.tri_idx = 0;
                self.face_idx += 1;
            }
            if tri.is_some() {
                return tri;
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Mesh triangulation
// ---------------------------------------------------------------------------

impl Mesh {
    /// Lazy iterator over the triangles of this mesh.
    ///
    /// Triangulates quads (4-vertex faces) into two triangles via fan
    /// triangulation (0,1,2) + (0,2,3).  Triangles (3 vertices) are
    /// yielded as-is. Faces with fewer than 3 vertices are skipped.
    pub fn iter_tris(&self) -> MeshTriIter<'_> {
        let face_keys: Vec<&String> = self.faces.keys().collect();
        MeshTriIter {
            mesh: self,
            face_keys,
            key_idx: 0,
            tri_rem: 0,
        }
    }
}

/// Iterator over triangles generated from a [`Mesh`]'s faces.
pub struct MeshTriIter<'a> {
    mesh: &'a Mesh,
    face_keys: Vec<&'a String>,
    key_idx: usize,
    tri_rem: usize,
}

impl<'a> Iterator for MeshTriIter<'a> {
    type Item = Tri;

    fn next(&mut self) -> Option<Tri> {
        loop {
            if self.tri_rem > 0 {
                let key = self.face_keys[self.key_idx - 1];
                let face = self.mesh.faces.get(key)?;
                let vcount = face.vertices.len();
                if vcount < 3 {
                    self.tri_rem = 0;
                    continue;
                }

                let pos: Vec<Vec3> = face
                    .vertices
                    .iter()
                    .filter_map(|vid| self.mesh.vertices.get(vid).copied())
                    .collect();
                if pos.len() != vcount {
                    self.tri_rem = 0;
                    continue;
                }

                let uvs: Vec<Vec2> = face
                    .vertices
                    .iter()
                    .map(|vid| face.uv.get(vid).copied().unwrap_or_else(|| v2(0.0, 0.0)))
                    .collect();

                let normal = Mesh::tri_normal(&pos[0], &pos[1], &pos[2]);

                // fan triangulation for quads (vcount==4): (0,1,2) then (0,2,3)
                // for triangles (vcount==3): (0,1,2)
                let (i0, i1, i2) = if vcount == 3 {
                    (0, 1, 2)
                } else {
                    // vcount == 4, tri_rem indicates which triangle
                    if self.tri_rem == 2 {
                        (0, 1, 2) // first tri of quad
                    } else {
                        (0, 2, 3) // second tri of quad
                    }
                };

                self.tri_rem -= 1;

                return Some(Tri {
                    positions: [pos[i0], pos[i1], pos[i2]],
                    uvs: [uvs[i0], uvs[i1], uvs[i2]],
                    normals: [normal, normal, normal],
                    texture: face.texture,
                });
            }

            if self.key_idx >= self.face_keys.len() {
                return None;
            }
            let key = self.face_keys[self.key_idx];
            let face = self.mesh.faces.get(key)?;
            let vcount = face.vertices.len();

            self.key_idx += 1;

            if vcount == 3 {
                self.tri_rem = 1;
            } else if vcount == 4 {
                self.tri_rem = 2;
            } else {
                self.tri_rem = 0;
            }
        }
    }
}

impl Mesh {
    /// Compute the unit normal of a triangle (p0, p1, p2) in CCW winding.
    fn tri_normal(p0: &Vec3, p1: &Vec3, p2: &Vec3) -> Vec3 {
        let ax = v3_x(p1) - v3_x(p0);
        let ay = v3_y(p1) - v3_y(p0);
        let az = v3_z(p1) - v3_z(p0);

        let bx = v3_x(p2) - v3_x(p0);
        let by = v3_y(p2) - v3_y(p0);
        let bz = v3_z(p2) - v3_z(p0);

        let nx = ay * bz - az * by;
        let ny = az * bx - ax * bz;
        let nz = ax * by - ay * bx;

        let len = sqrt(nx * nx + ny * ny + nz * nz);
        if len > 1e-10 {
            v3(nx / len, ny / len, nz / len)
        } else {
            v3(0.0, 1.0, 0.0)
        }
    }
}
