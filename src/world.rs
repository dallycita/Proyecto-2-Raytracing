use crate::material::Material;
use crate::noise::{fbm, hash};
use crate::vec3::Vec3;

// tamaño del diorama en bloques
pub const W: i32 = 24;
pub const H: i32 = 18;
pub const D: i32 = 24;

// ids de los bloques (mismo orden que en material.rs)
pub const AIR: u8 = 0;
pub const GRASS: u8 = 1;
pub const DIRT: u8 = 2;
pub const STONE: u8 = 3;
pub const SAND: u8 = 4;
pub const WATER: u8 = 5;
pub const WOOD: u8 = 6;
pub const LEAVES: u8 = 7;
pub const ICE: u8 = 8;
pub const GLOWSTONE: u8 = 9;

// info de donde choco un rayo
pub struct Hit {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub block: u8,
    pub cell: [i32; 3],
}

pub struct World {
    blocks: Vec<u8>,
}

impl World {
    pub fn new(seed: u32) -> World {
        let mut world = World {
            blocks: vec![AIR; (W * H * D) as usize],
        };
        world.generate(seed);
        world
    }

    fn index(x: i32, y: i32, z: i32) -> usize {
        ((y * D + z) * W + x) as usize
    }

    pub fn inside(x: i32, y: i32, z: i32) -> bool {
        x >= 0 && x < W && y >= 0 && y < H && z >= 0 && z < D
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> u8 {
        if World::inside(x, y, z) {
            self.blocks[World::index(x, y, z)]
        } else {
            AIR
        }
    }

    fn set(&mut self, x: i32, y: i32, z: i32, block: u8) {
        if World::inside(x, y, z) {
            self.blocks[World::index(x, y, z)] = block;
        }
    }

    // ---------- generacion del terreno ----------

    fn generate(&mut self, seed: u32) {
        // primero saco la altura de cada columna con ruido
        let mut heights = vec![0i32; (W * D) as usize];
        for z in 0..D {
            for x in 0..W {
                let n = fbm(x as f32 * 0.09, z as f32 * 0.09, seed, 4);
                // le subo el contraste para que haya montañas y valles
                let n = ((n - 0.5) * 1.9 + 0.5).clamp(0.0, 1.0);
                heights[(z * W + x) as usize] = 1 + (n * 11.0) as i32;
            }
        }

        let min_h = *heights.iter().min().unwrap();
        let max_h = *heights.iter().max().unwrap();
        // el agua depende de lo mas bajo, asi siempre sale un lago
        let water = min_h + 2;
        let mountain = water + 5;

        for z in 0..D {
            for x in 0..W {
                let h = heights[(z * W + x) as usize];
                for y in 0..=h {
                    let block = if y == h {
                        // bloque de hasta arriba
                        if h <= water + 1 {
                            SAND
                        } else if h >= max_h - 1 && h >= mountain {
                            ICE // cima congelada
                        } else if h >= mountain {
                            STONE
                        } else {
                            GRASS
                        }
                    } else if y >= h - 2 {
                        // las capas de abajo del bloque de arriba
                        if h <= water + 1 {
                            SAND
                        } else if h >= mountain {
                            STONE
                        } else {
                            DIRT
                        }
                    } else {
                        STONE
                    };
                    self.set(x, y, z, block);
                }
                // relleno de agua hasta el nivel del lago
                for y in (h + 1)..=water {
                    self.set(x, y, z, WATER);
                }
            }
        }

        self.place_decorations(seed, &heights);
    }

    fn place_decorations(&mut self, seed: u32, heights: &[i32]) {
        // busco columnas con pasto que no esten en el borde
        let mut spots: Vec<(i32, i32)> = Vec::new();
        for z in 2..D - 2 {
            for x in 2..W - 2 {
                let h = heights[(z * W + x) as usize];
                if self.get(x, h, z) == GRASS {
                    spots.push((x, z));
                }
            }
        }

        // las revuelvo con la semilla para que cambien en cada terreno
        spots.sort_by(|a, b| {
            hash(a.0, a.1, seed)
                .partial_cmp(&hash(b.0, b.1, seed))
                .unwrap()
        });

        let mut used: Vec<(i32, i32)> = Vec::new();
        let mut lamps = 0;
        let mut trees = 0;
        for &(x, z) in &spots {
            // que no queden pegados unos con otros
            let far = used
                .iter()
                .all(|&(ux, uz)| (ux - x).abs() + (uz - z).abs() >= 5);
            if !far {
                continue;
            }
            let h = heights[(z * W + x) as usize];
            if lamps < 5 {
                self.place_lamp(x, h, z);
                lamps += 1;
            } else if trees < 7 {
                self.place_tree(x, h, z, seed);
                trees += 1;
            } else {
                break;
            }
            used.push((x, z));
        }
    }

    // poste de madera con glowstone arriba
    fn place_lamp(&mut self, x: i32, h: i32, z: i32) {
        self.set(x, h + 1, z, WOOD);
        self.set(x, h + 2, z, GLOWSTONE);
    }

    fn place_tree(&mut self, x: i32, h: i32, z: i32, seed: u32) {
        let trunk = 3 + (hash(x, z, seed.wrapping_add(1)) * 2.0) as i32;
        for y in 1..=trunk {
            self.set(x, h + y, z, WOOD);
        }

        // hojas alrededor de la punta del tronco
        let top = h + trunk;
        for dy in -1..=1 {
            for dz in -2..=2 {
                for dx in -2..=2 {
                    if dx.abs() == 2 && dz.abs() == 2 {
                        continue; // sin esquinas para que se vea mas redondo
                    }
                    if self.get(x + dx, top + dy, z + dz) == AIR {
                        self.set(x + dx, top + dy, z + dz, LEAVES);
                    }
                }
            }
        }
        // una crucecita de hojas hasta arriba
        for dz in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 || dz == 0 {
                    self.set(x + dx, top + 2, z + dz, LEAVES);
                }
            }
        }
    }

    // centro de cada glowstone, se usan como luces
    pub fn lamps(&self) -> Vec<Vec3> {
        let mut list = Vec::new();
        for y in 0..H {
            for z in 0..D {
                for x in 0..W {
                    if self.get(x, y, z) == GLOWSTONE {
                        list.push(Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5));
                    }
                }
            }
        }
        list
    }

    // ---------- recorrido de rayos ----------

    // avanza el rayo bloque por bloque (DDA en 3D) hasta que stop() diga que pare
    // asi no hay que probar el rayo contra todos los cubos
    pub fn march<F: Fn(u8) -> bool>(&self, origin: Vec3, dir: Vec3, max_t: f32, stop: F) -> Option<Hit> {
        // si el rayo ni toca la caja del mundo, ni lo recorro
        let (t_start, entry_axis) = clip_to_world(origin, dir)?;
        if t_start > max_t {
            return None;
        }

        let base = t_start + 1e-4;
        let p = origin + dir * base;

        let mut x = (p.x.floor() as i32).clamp(0, W - 1);
        let mut y = (p.y.floor() as i32).clamp(0, H - 1);
        let mut z = (p.z.floor() as i32).clamp(0, D - 1);

        let step_x = if dir.x > 0.0 { 1 } else { -1 };
        let step_y = if dir.y > 0.0 { 1 } else { -1 };
        let step_z = if dir.z > 0.0 { 1 } else { -1 };

        // cuanto hay que avanzar para cruzar un bloque entero en cada eje
        let delta_x = if dir.x != 0.0 { (1.0 / dir.x).abs() } else { f32::MAX };
        let delta_y = if dir.y != 0.0 { (1.0 / dir.y).abs() } else { f32::MAX };
        let delta_z = if dir.z != 0.0 { (1.0 / dir.z).abs() } else { f32::MAX };

        // distancia hasta la primera pared de cada eje
        let mut side_x = first_side(p.x, x, dir.x);
        let mut side_y = first_side(p.y, y, dir.y);
        let mut side_z = first_side(p.z, z, dir.z);

        let mut t = 0.0;
        let mut axis = entry_axis;

        loop {
            let block = self.get(x, y, z);
            if stop(block) {
                // la normal apunta contra el rayo en el eje que cruzamos
                let normal = match axis {
                    0 => Vec3::new(-step_x as f32, 0.0, 0.0),
                    1 => Vec3::new(0.0, -step_y as f32, 0.0),
                    _ => Vec3::new(0.0, 0.0, -step_z as f32),
                };
                let total = base + t;
                return Some(Hit {
                    t: total,
                    point: origin + dir * total,
                    normal,
                    block,
                    cell: [x, y, z],
                });
            }

            // paso al siguiente bloque por el eje mas cercano
            if side_x < side_y && side_x < side_z {
                x += step_x;
                t = side_x;
                side_x += delta_x;
                axis = 0;
            } else if side_y < side_z {
                y += step_y;
                t = side_y;
                side_y += delta_y;
                axis = 1;
            } else {
                z += step_z;
                t = side_z;
                side_z += delta_z;
                axis = 2;
            }

            if !World::inside(x, y, z) || base + t > max_t {
                return None;
            }
        }
    }

    // true si algo tapa la luz en esa direccion
    pub fn in_shadow(&self, origin: Vec3, dir: Vec3, max_t: f32, materials: &[Material]) -> bool {
        let hit = self.march(origin, dir, max_t, |b| {
            if b == AIR {
                return false;
            }
            let m = &materials[b as usize];
            // el agua, el hielo y las lamparas dejan pasar la luz
            m.transparency == 0.0 && m.emission == 0.0
        });
        hit.is_some()
    }
}

fn first_side(pos: f32, cell: i32, dir: f32) -> f32 {
    if dir > 0.0 {
        (cell as f32 + 1.0 - pos) / dir
    } else if dir < 0.0 {
        (pos - cell as f32) / -dir
    } else {
        f32::MAX
    }
}

// interseccion del rayo con la caja que encierra todo el mundo (metodo de slabs)
// devuelve donde entra y por cual eje entro
fn clip_to_world(o: Vec3, d: Vec3) -> Option<(f32, usize)> {
    let mut t0 = 0.0;
    let mut t1 = f32::MAX;
    let mut axis = 1;

    for i in 0..3 {
        let (oi, di, max) = match i {
            0 => (o.x, d.x, W as f32),
            1 => (o.y, d.y, H as f32),
            _ => (o.z, d.z, D as f32),
        };
        if di.abs() < 1e-8 {
            // rayo paralelo a este eje, tiene que estar adentro
            if oi < 0.0 || oi > max {
                return None;
            }
            continue;
        }
        let mut ta = (0.0 - oi) / di;
        let mut tb = (max - oi) / di;
        if ta > tb {
            std::mem::swap(&mut ta, &mut tb);
        }
        if ta > t0 {
            t0 = ta;
            axis = i;
        }
        if tb < t1 {
            t1 = tb;
        }
        if t0 > t1 {
            return None;
        }
    }
    Some((t0, axis))
}
