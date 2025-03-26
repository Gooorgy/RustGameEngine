use crate::terrain::blocks::blocks::BlockType;
use crate::terrain::constants::{CHUNK_SIZE, CHUNK_STORAGE_SIZE};
use crate::terrain::terrain_material::VoxelData;
use noise::{Fbm, MultiFractal, NoiseFn, Simplex};

const BASE_HEIGHT: usize = 5;

pub fn new_terrain(seed: u32) -> [VoxelData; CHUNK_STORAGE_SIZE] {
    let fbm_simplex = Fbm::<Simplex>::new(seed).set_octaves(5).set_frequency(0.01);
    let mut out = [VoxelData::default(); CHUNK_STORAGE_SIZE];

    // Fill entire chunk space including padding
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let index = x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE;

                if y < BASE_HEIGHT {
                    out[index] = VoxelData::from_block_name_space(BlockType::GRASS.as_namespace());
                    continue;
                }

                // Use actual coordinates for noise generation
                let noise_x = x as f64;
                let noise_y = y as f64;
                let noise_z = z as f64;
                let density = fbm_simplex.get([noise_x, noise_y, noise_z]);

                let delta = BASE_HEIGHT as f64 / y as f64;
                let scaled_noise_val = scale(density, -1.0, 1.0, 0.0, 1.0) * delta;

                if scaled_noise_val > 0.4 {
                    out[index] = VoxelData::from_block_name_space(BlockType::GRASS.as_namespace());
                }
            }
        }
    }

    out
}

fn scale(val: f64, min: f64, max: f64, new_min: f64, new_max: f64) -> f64 {
    let percentage = (val - min) / (min - max);
    percentage * (new_min - new_max) + new_min
}
