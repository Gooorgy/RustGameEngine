use glm::{Vec2, Vec3};
use vulkan_backend::render_objects::draw_objects::Vertex;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_STORAGE_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub const VOXEL_SIZE: usize = 20;
pub const VOXEL_SIZE_I32: i32 = VOXEL_SIZE as i32;
//pub const VOXEL_SIZE_F32: f32 = VOXEL_SIZE as f32;

pub const VOXEL_SIZE_HALF: usize = VOXEL_SIZE / 2;
pub const VOXEL_SIZE_HALF_F32: f32 = VOXEL_SIZE_HALF as f32;

const NORMAL_TOP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
const NORMAL_BOTTOM: Vec3 = Vec3::new(0.0, -1.0, 0.0);
const NORMAL_LEFT: Vec3 = Vec3::new(-1.0, 0.0, 0.0);
const NORMAL_RIGHT: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const NORMAL_FRONT: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const NORMAL_BACK: Vec3 = Vec3::new(0.0, 0.0, -1.0);

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum Face {
    Bottom = 0usize,
    Top = 1usize,
    Back = 2usize,
    Front = 3usize,
    Left = 4usize,
    Right = 5usize,
}

impl Face {
    pub fn iter() -> impl Iterator<Item = Face> {
        [
            Self::Bottom,
            Self::Top,
            Self::Back,
            Self::Front,
            Self::Left,
            Self::Right,
        ]
        .iter()
        .cloned()
    }
}

pub const FACE_INDICES: [[u32; 6]; 6] = [
    [0, 1, 2, 2, 3, 0], // Bottom
    [2, 1, 0, 0, 3, 2], // Top
    [2, 1, 0, 0, 3, 2], // Back
    [0, 1, 2, 2, 3, 0], // Front
    [0, 1, 2, 2, 3, 0], // Left
    [2, 1, 0, 0, 3, 2], // Right
];

pub const FACE_VERTICES: [[Vertex; 4]; 6] = [
    [
        // Bottom Face
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 0.0),
            normal: NORMAL_BOTTOM,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 0.0),
            normal: NORMAL_BOTTOM,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 1.0),
            normal: NORMAL_BOTTOM,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 1.0),
            normal: NORMAL_BOTTOM,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
    ],
    [
        // Top Face
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 0.0),
            normal: NORMAL_TOP,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 0.0),
            normal: NORMAL_TOP,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 1.0),
            normal: NORMAL_TOP,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 1.0),
            normal: NORMAL_TOP,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
    ],
    [
        // Back Face
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 0.0),
            normal: NORMAL_BACK,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 0.0),
            normal: NORMAL_BACK,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 1.0),
            normal: NORMAL_BACK,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 1.0),
            normal: NORMAL_BACK,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
    ],
    [
        // Front Face
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 0.0),
            normal: NORMAL_FRONT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 0.0),
            normal: NORMAL_FRONT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 1.0),
            normal: NORMAL_FRONT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 1.0),
            normal: NORMAL_FRONT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
    ],
    [
        // Left Face
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 0.0),
            normal: NORMAL_LEFT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 0.0),
            normal: NORMAL_LEFT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 1.0),
            normal: NORMAL_LEFT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 1.0),
            normal: NORMAL_LEFT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
    ],
    [
        // Right Face
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 0.0),
            normal: NORMAL_RIGHT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 0.0),
            normal: NORMAL_RIGHT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                -VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(1.0, 1.0),
            normal: NORMAL_RIGHT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
        Vertex {
            pos: Vec3::new(
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
                VOXEL_SIZE_HALF_F32,
            ),
            tex_coord: Vec2::new(0.0, 1.0),
            normal: NORMAL_RIGHT,
            color: Vec3::new(0.0, 0.0, 0.0),
            texture_index: 0,
        },
    ],
];
