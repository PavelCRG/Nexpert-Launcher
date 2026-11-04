#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
}

pub struct World {
    blocks: Vec<Block>,
}

#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Default)]
pub struct WorldMesh {
    pub grass_top: Vec<Vertex>,
    pub grass_side: Vec<Vertex>,
    pub dirt: Vec<Vertex>,
    pub stone: Vec<Vertex>,
}

impl WorldMesh {
    #[cfg(test)]
    pub fn total_vertices(&self) -> usize {
        self.grass_top.len() + self.grass_side.len() + self.dirt.len() + self.stone.len()
    }
}

impl World {
    const WIDTH: usize = 16;
    const DEPTH: usize = 16;
    const HEIGHT: usize = 8;

    pub fn starter(seed: u64) -> Self {
        let mut blocks = vec![Block::Air; Self::WIDTH * Self::DEPTH * Self::HEIGHT];
        for z in 0..Self::DEPTH {
            for x in 0..Self::WIDTH {
                let top = 2 + (height_hash(seed, x, z) % 3) as usize;
                for y in 0..=top {
                    let block = if y == top {
                        Block::Grass
                    } else if y + 2 >= top {
                        Block::Dirt
                    } else {
                        Block::Stone
                    };
                    blocks[index(x, y, z)] = block;
                }
            }
        }
        Self { blocks }
    }

    pub fn block(&self, x: usize, y: usize, z: usize) -> Block {
        if x >= Self::WIDTH || y >= Self::HEIGHT || z >= Self::DEPTH {
            return Block::Air;
        }
        self.blocks[index(x, y, z)]
    }

    #[cfg(test)]
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    #[cfg(test)]
    pub fn width(&self) -> usize {
        Self::WIDTH
    }

    #[cfg(test)]
    pub fn depth(&self) -> usize {
        Self::DEPTH
    }

    #[cfg(test)]
    pub fn height(&self) -> usize {
        Self::HEIGHT
    }

    pub fn mesh(&self) -> WorldMesh {
        let mut mesh = WorldMesh::default();
        for y in 0..Self::HEIGHT {
            for z in 0..Self::DEPTH {
                for x in 0..Self::WIDTH {
                    let block = self.block(x, y, z);
                    if block == Block::Air {
                        continue;
                    }

                    let x0 = x as f32;
                    let y0 = y as f32;
                    let z0 = z as f32;
                    let x1 = x0 + 1.0;
                    let y1 = y0 + 1.0;
                    let z1 = z0 + 1.0;
                    let faces = [
                        (
                            0,
                            self.block(x, y + 1, z),
                            [[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]],
                        ),
                        (
                            1,
                            self.block(x, y, z + 1),
                            [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]],
                        ),
                        (
                            1,
                            self.block(x, y, z.wrapping_sub(1)),
                            [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]],
                        ),
                        (
                            1,
                            self.block(x + 1, y, z),
                            [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]],
                        ),
                        (
                            1,
                            self.block(x.wrapping_sub(1), y, z),
                            [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]],
                        ),
                        (
                            2,
                            self.block(x, y.wrapping_sub(1), z),
                            [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]],
                        ),
                    ];

                    for (face, neighbor, corners) in faces {
                        if neighbor != Block::Air {
                            continue;
                        }
                        let target = match (block, face) {
                            (Block::Grass, 0) => &mut mesh.grass_top,
                            (Block::Grass, _) => &mut mesh.grass_side,
                            (Block::Dirt, _) => &mut mesh.dirt,
                            (Block::Stone, _) => &mut mesh.stone,
                            (Block::Air, _) => unreachable!(),
                        };
                        append_quad(target, corners);
                    }
                }
            }
        }
        mesh
    }
}

fn append_quad(target: &mut Vec<Vertex>, corners: [[f32; 3]; 4]) {
    const UVS: [[f32; 2]; 4] = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    for index in [0, 1, 2, 0, 2, 3] {
        target.push(Vertex {
            position: corners[index],
            uv: UVS[index],
        });
    }
}

fn index(x: usize, y: usize, z: usize) -> usize {
    (y * World::DEPTH + z) * World::WIDTH + x
}

fn height_hash(seed: u64, x: usize, z: usize) -> u64 {
    let mut value = seed ^ ((x as u64) << 32) ^ z as u64;
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{Block, World};

    #[test]
    fn starter_world_is_deterministic_for_a_seed() {
        let first = World::starter(42);
        let second = World::starter(42);
        assert_eq!(first.blocks(), second.blocks());
    }

    #[test]
    fn starter_world_has_grass_surface_and_air_above_it() {
        let world = World::starter(42);
        let mut found_surface = false;
        for z in 0..world.depth() {
            for x in 0..world.width() {
                for y in (0..world.height()).rev() {
                    if world.block(x, y, z) != Block::Air {
                        assert_eq!(world.block(x, y, z), Block::Grass);
                        assert_eq!(world.block(x, y + 1, z), Block::Air);
                        found_surface = true;
                        break;
                    }
                }
            }
        }
        assert!(found_surface);
    }

    #[test]
    fn mesh_contains_only_exposed_block_faces() {
        let world = World::starter(42);
        let mesh = world.mesh();
        let solid_blocks = world
            .blocks()
            .iter()
            .filter(|block| **block != Block::Air)
            .count();
        let maximum_unoptimized_vertices = solid_blocks * 36;

        assert!(mesh.grass_top.len() >= world.width() * world.depth() * 6);
        assert!(mesh.total_vertices() < maximum_unoptimized_vertices);
        assert_eq!(mesh.total_vertices() % 6, 0);
    }
}
