use crate::vulkan_render::scene::Mesh;
use crate::vulkan_render::structs::Vertex;
use glm::{vec2, vec3, Vec3};
use nalgebra::Vector3;
use noise::{Fbm, MultiFractal, NoiseFn, Simplex};

const VOXEL_SIZE: i32 = 20;
const VOXEL_SIZE_HALF: i32 = VOXEL_SIZE / 2;
const BASE_HEIGHT: u32 = 10;

const NORMAL_TOP: Vec3 =Vector3::new(0.0f32, 1.0f32, 0.0f32);
const NORMAL_BOTTOM: Vec3 = Vector3::new(0.0, -1.0, 0.0);
const NORMAL_LEFT: Vec3 = Vector3::new(-1.0, 0.0, 0.0);
const NORMAL_RIGHT: Vec3 = Vector3::new(1.0, 0.0, 0.0);
const NORMAL_FRONT: Vec3 = Vector3::new(0.0, 0.0, 1.0);
const NORMAL_BACK: Vec3 = Vector3::new(0.0, 0.0, -1.0);


pub fn new_terrain(seed: u32, size: u32) -> Vec<Vec<Vec<u32>>> {
    let fbm_simplex = Fbm::<Simplex>::new(seed).set_octaves(5).set_frequency(0.01);
    let mut out = vec![vec![vec![0; size as usize]; size as usize]; size as usize];
    size as usize;
    for x in 0..size {

        for y in 0..50 {
            for z in 0..size {
                if y < BASE_HEIGHT {
                    out[x as usize][y as usize][z as usize] = 1;
                    continue;
                }

                let delta = BASE_HEIGHT as f64 / y as f64;

                let density = fbm_simplex.get([x as f64, y as f64, z as f64]);

                let scaled_noise_val = scale(density, -1.0, 1.0, 0.0, 1.0) * delta;
                if scaled_noise_val > 0.4 {
                    out[x as usize][y as usize][z as usize] = 1
                }
            }
        }
    }

    out
}

pub fn generate_mesh(terrain_data: Vec<Vec<Vec<u32>>>) -> Mesh {
    let mut vertices = vec![];
    let mut indices = vec![];
    let mut element_index: u32 = 0;
    for x in 1..terrain_data.len() - 1 {
        for y in 0..terrain_data[x].len() - 1 {
            for z in 1..terrain_data[x][y].len() - 1 {
                if terrain_data[x][y][z] <= 0 {
                    continue;
                }

                for f in 0..6 {
                    match f {
                        0 => {
                            // Top face
                            if terrain_data[x][y + 1][z] == 0 || y + 1 == terrain_data.len() {
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 255.0, 0.0),
                                    tex_coord: vec2(0.0, 0.0),
                                    normal: NORMAL_TOP,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 255.0, 0.0),
                                    tex_coord: vec2(1.0, 0.0),
                                    normal: NORMAL_TOP,

                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 255.0, 0.0),
                                    tex_coord: vec2(1.0, 1.0),
                                    normal: NORMAL_TOP,

                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 255.0, 0.0),
                                    tex_coord: vec2(0.0, 1.0),
                                    normal: NORMAL_TOP,
                                });

                                indices.push(2 + element_index);
                                indices.push(1 + element_index);
                                indices.push(0 + element_index);
                                indices.push(0 + element_index);
                                indices.push(3 + element_index);
                                indices.push(2 + element_index);

                                element_index += 4;
                            }
                        }
                        // Bottom face
                        1 => {
                            if y as i32 - 1 == -1 || terrain_data[x][y - 1][z] == 0 {
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(255.0, 0.0, 0.0),
                                    tex_coord: vec2(0.0, 0.0),
                                    normal: NORMAL_BOTTOM,

                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(255.0, 0.0, 0.0),
                                    tex_coord: vec2(0.0, 1.0),
                                    normal: NORMAL_BOTTOM,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(255.0, 0.0, 0.0),
                                    tex_coord: vec2(1.0, 1.0),
                                    normal: NORMAL_BOTTOM,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(255.0, 0.0, 0.0),
                                    tex_coord: vec2(1.0, 0.0),
                                    normal: NORMAL_BOTTOM,
                                });

                                indices.push(0 + element_index);
                                indices.push(1 + element_index);
                                indices.push(2 + element_index);

                                indices.push(2 + element_index);
                                indices.push(3 + element_index);
                                indices.push(0 + element_index);

                                element_index += 4;
                            }
                        }
                        // Left face
                        2 => {
                            if terrain_data[x - 1][y][z] == 0 {
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(0.0, 0.0),
                                    normal: NORMAL_LEFT,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(0.0, 1.0),
                                    normal: NORMAL_LEFT,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(1.0, 1.0),
                                    normal: NORMAL_LEFT,

                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(1.0, 0.0),
                                    normal: NORMAL_LEFT,
                                });

                                indices.push(0 + element_index);
                                indices.push(1 + element_index);
                                indices.push(2 + element_index);

                                indices.push(2 + element_index);
                                indices.push(3 + element_index);
                                indices.push(0 + element_index);
                                element_index += 4;
                            }
                        }
                        3 => {
                            if terrain_data[x + 1][y][z] == 0 {
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(0.0, 0.0),
                                    normal: NORMAL_RIGHT,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(0.0, 1.0),
                                    normal: NORMAL_RIGHT,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(1.0, 1.0),
                                    normal: NORMAL_RIGHT,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(1.0, 0.0),
                                    normal: NORMAL_RIGHT,
                                });

                                indices.push(2 + element_index);
                                indices.push(1 + element_index);
                                indices.push(0 + element_index);

                                indices.push(0 + element_index);
                                indices.push(3 + element_index);
                                indices.push(2 + element_index);
                                element_index += 4;
                            }
                        }
                        4 => {
                            if terrain_data[x][y][z - 1] == 0 {
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(0.0, 0.0),
                                    normal: NORMAL_BACK,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(0.0, 1.0),
                                    normal: NORMAL_BACK,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(1.0, 1.0),
                                    normal: NORMAL_BACK,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(1.0, 0.0),
                                    normal: NORMAL_BACK,
                                });

                                indices.push(2 + element_index);
                                indices.push(1 + element_index);
                                indices.push(0 + element_index);

                                indices.push(0 + element_index);
                                indices.push(3 + element_index);
                                indices.push(2 + element_index);
                                element_index += 4;
                            }
                        }
                        5 => {
                            if terrain_data[x][y][z + 1] == 0 {
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(0.0, 0.0),
                                    normal: NORMAL_FRONT,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (-1 * VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(0.0, 1.0),
                                    normal: NORMAL_FRONT,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (-1 * VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(1.0, 1.0),
                                    normal: NORMAL_FRONT,
                                });
                                vertices.push(Vertex {
                                    pos: vec3(
                                        (VOXEL_SIZE_HALF + (x as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (y as i32 * VOXEL_SIZE)) as f32,
                                        (VOXEL_SIZE_HALF + (z as i32 * VOXEL_SIZE)) as f32,
                                    ),
                                    color: vec3(0.0, 0.0, 0.0),
                                    tex_coord: vec2(1.0, 0.0),
                                    normal: NORMAL_FRONT,
                                });

                                indices.push(0 + element_index);
                                indices.push(1 + element_index);
                                indices.push(2 + element_index);

                                indices.push(2 + element_index);
                                indices.push(3 + element_index);
                                indices.push(0 + element_index);
                                element_index += 4;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Mesh { vertices, indices }
}

fn scale(val: f64, min: f64, max: f64, new_min: f64, new_max: f64) -> f64 {
    let percentage = (val - min) / (min - max);
    percentage * (new_min - new_max) + new_min
}
