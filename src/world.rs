use crate::material::Material;
use crate::noise::{fbm, hash};
use crate::vec3::Vec3;

// tamaño del diorama en bloques
pub const W: i32 = 32;
pub const H: i32 = 24;
pub const D: i32 = 32;

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
pub const PLANKS: u8 = 10;
pub const GLASS: u8 = 11;
pub const LAVA: u8 = 12;
pub const COBBLE: u8 = 13;

// info de donde choco un rayo
pub struct Hit {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub block: u8,
    pub cell: [i32; 3],
}

// luz puntual que sale de un bloque emisivo
pub struct Light {
    pub pos: Vec3,
    pub cell: [i32; 3],
    pub color: Vec3,
    pub power: f32,
}

pub struct World {
    blocks: Vec<u8>,
}

// indice de una columna (x, z) en los arreglos de 2D
fn col(x: i32, z: i32) -> usize {
    (z * W + x) as usize
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
                let n = fbm(x as f32 * 0.07, z as f32 * 0.07, seed, 5);
                // le subo el contraste para que haya montañas y valles
                let n = ((n - 0.5) * 2.0 + 0.5).clamp(0.0, 1.0);
                heights[col(x, z)] = 1 + (n * 14.0) as i32;
            }
        }

        let min_h = *heights.iter().min().unwrap();
        let max_h = *heights.iter().max().unwrap();
        // el agua depende de lo mas bajo, asi siempre sale un lago
        let water = min_h + 2;
        let mountain = water + 6;

        for z in 0..D {
            for x in 0..W {
                self.fill_column(x, z, heights[col(x, z)], water, mountain, max_h);
            }
        }

        // lugares ya usados, para que las cosas no se encimen
        let mut blocked = vec![false; (W * D) as usize];
        self.place_house(seed, &mut heights, &mut blocked, water, mountain);
        self.place_pier(seed, &heights, &mut blocked, water);
        self.place_lava(seed, &heights, &mut blocked);
        self.place_nature(seed, &heights, &mut blocked);
    }

    fn fill_column(&mut self, x: i32, z: i32, h: i32, water: i32, mountain: i32, max_h: i32) {
        let beach = h <= water + 1;
        let rocky = h >= mountain;
        for y in 0..=h {
            let block = if y == h {
                // bloque de hasta arriba
                if beach {
                    SAND
                } else if rocky && h >= max_h - 2 {
                    ICE // cimas congeladas
                } else if rocky {
                    STONE
                } else {
                    GRASS
                }
            } else if y >= h - 2 {
                if beach {
                    SAND
                } else if rocky {
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

    // casita de 5x5 con ventanas de vidrio, chimenea y una lampara adentro
    fn place_house(&mut self, seed: u32, heights: &mut [i32], blocked: &mut [bool], water: i32, mountain: i32) {
        // busco el lugar mas plano que no este en la playa ni en la montaña
        let mut best: Option<(i32, i32, i32)> = None;
        let mut best_score = f32::MAX;
        for z0 in 3..D - 8 {
            for x0 in 3..W - 8 {
                let mut lo = i32::MAX;
                let mut hi = i32::MIN;
                for dz in 0..5 {
                    for dx in 0..5 {
                        let h = heights[col(x0 + dx, z0 + dz)];
                        lo = lo.min(h);
                        hi = hi.max(h);
                    }
                }
                if lo <= water + 1 || hi >= mountain - 1 {
                    continue;
                }
                let score = (hi - lo) as f32 + hash(x0, z0, seed.wrapping_add(7)) * 0.9;
                if score < best_score {
                    best_score = score;
                    best = Some((x0, z0, (lo + hi) / 2));
                }
            }
        }
        let Some((x0, z0, h)) = best else { return };

        // aplano el terreno (con un bloque extra alrededor)
        for z in (z0 - 1)..=(z0 + 5) {
            for x in (x0 - 1)..=(x0 + 5) {
                let floor = x >= x0 && x <= x0 + 4 && z >= z0 && z <= z0 + 4;
                for y in 0..H {
                    let block = if y < h - 2 {
                        STONE
                    } else if y < h {
                        DIRT
                    } else if y == h {
                        if floor { COBBLE } else { GRASS }
                    } else {
                        AIR
                    };
                    self.set(x, y, z, block);
                }
                heights[col(x, z)] = h;
            }
        }

        // paredes: troncos en las esquinas y tablas en lo demas
        for dz in 0..5 {
            for dx in 0..5 {
                let edge_x = dx == 0 || dx == 4;
                let edge_z = dz == 0 || dz == 4;
                if !(edge_x || edge_z) {
                    continue;
                }
                let block = if edge_x && edge_z { WOOD } else { PLANKS };
                for y in (h + 1)..=(h + 3) {
                    self.set(x0 + dx, y, z0 + dz, block);
                }
            }
        }

        // ventanas
        self.set(x0 + 2, h + 2, z0 + 4, GLASS);
        self.set(x0, h + 2, z0 + 2, GLASS);
        self.set(x0 + 4, h + 2, z0 + 2, GLASS);

        // puerta
        self.set(x0 + 2, h + 1, z0, AIR);
        self.set(x0 + 2, h + 2, z0, AIR);

        // techo en forma de piramide
        for layer in 0..4 {
            let r = 3 - layer;
            for dz in -r..=r {
                for dx in -r..=r {
                    self.set(x0 + 2 + dx, h + 4 + layer, z0 + 2 + dz, PLANKS);
                }
            }
        }

        // chimenea
        for y in (h + 4)..=(h + 8) {
            self.set(x0 + 3, y, z0 + 3, COBBLE);
        }

        // lampara colgando del techo por dentro
        self.set(x0 + 2, h + 3, z0 + 2, GLOWSTONE);

        mark(blocked, x0 + 2, z0 + 2, 4);
    }

    // muelle de tablas desde la playa hacia el lago
    fn place_pier(&mut self, seed: u32, heights: &[i32], blocked: &mut [bool], water: i32) {
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let mut best: Option<(i32, i32, i32, i32, i32)> = None; // x, z, dx, dz, largo
        let mut best_score = 0.0;

        for z in 2..D - 2 {
            for x in 2..W - 2 {
                if self.get(x, heights[col(x, z)], z) != SAND || blocked[col(x, z)] {
                    continue;
                }
                for &(dx, dz) in &dirs {
                    // cuantos bloques de agua hay seguidos en esa direccion
                    let mut len = 0;
                    while len < 6 {
                        let cx = x + dx * (len + 1);
                        let cz = z + dz * (len + 1);
                        if cx < 1 || cx >= W - 1 || cz < 1 || cz >= D - 1 {
                            break;
                        }
                        if heights[col(cx, cz)] >= water {
                            break;
                        }
                        len += 1;
                    }
                    if len < 3 {
                        continue;
                    }
                    let score = len as f32 + hash(x, z, seed.wrapping_add(11));
                    if score > best_score {
                        best_score = score;
                        best = Some((x, z, dx, dz, len));
                    }
                }
            }
        }
        let Some((x, z, dx, dz, len)) = best else { return };

        for i in 1..=len {
            let cx = x + dx * i;
            let cz = z + dz * i;
            self.set(cx, water + 1, cz, PLANKS);
            // postes que se hunden en el agua
            if i % 2 == 0 || i == len {
                for y in (heights[col(cx, cz)] + 1)..=water {
                    self.set(cx, y, cz, WOOD);
                }
            }
            mark(blocked, cx, cz, 1);
        }

        // lampara al final del muelle
        let ex = x + dx * len;
        let ez = z + dz * len;
        self.set(ex, water + 2, ez, WOOD);
        self.set(ex, water + 3, ez, GLOWSTONE);
        mark(blocked, x, z, 1);
    }

    // charquito de lava en la zona de piedra, mejor si esta en un hoyito
    fn place_lava(&mut self, seed: u32, heights: &[i32], blocked: &mut [bool]) {
        let mut best: Option<(i32, i32)> = None;
        let mut best_score = f32::MAX;
        for z in 2..D - 2 {
            for x in 2..W - 2 {
                let h = heights[col(x, z)];
                if self.get(x, h, z) != STONE || blocked[col(x, z)] {
                    continue;
                }
                let mut higher = 0;
                for &(dx, dz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if heights[col(x + dx, z + dz)] >= h {
                        higher += 1;
                    }
                }
                let score = (4 - higher) as f32 + hash(x, z, seed.wrapping_add(21));
                if score < best_score {
                    best_score = score;
                    best = Some((x, z));
                }
            }
        }
        let Some((x, z)) = best else { return };

        let h = heights[col(x, z)];
        self.set(x, h, z, LAVA);
        // si los vecinos estan a la misma altura el charco crece
        for &(dx, dz) in &[(1, 0), (0, 1), (1, 1)] {
            let cx = x + dx;
            let cz = z + dz;
            if heights[col(cx, cz)] == h && self.get(cx, h, cz) == STONE {
                self.set(cx, h, cz, LAVA);
            }
        }
        mark(blocked, x, z, 2);
    }

    // lamparas, arboles y arbustos sobre el pasto
    fn place_nature(&mut self, seed: u32, heights: &[i32], blocked: &mut [bool]) {
        let mut spots: Vec<(i32, i32)> = Vec::new();
        for z in 2..D - 2 {
            for x in 2..W - 2 {
                if self.get(x, heights[col(x, z)], z) == GRASS {
                    spots.push((x, z));
                }
            }
        }

        // los revuelvo con la semilla para que cambien en cada terreno
        spots.sort_by(|a, b| {
            hash(a.0, a.1, seed)
                .partial_cmp(&hash(b.0, b.1, seed))
                .unwrap()
        });

        let mut lamps = 0;
        let mut trees = 0;
        for &(x, z) in &spots {
            let h = heights[col(x, z)];
            if lamps < 4 && area_free(blocked, x, z, 3) {
                self.place_lamp(x, h, z);
                mark(blocked, x, z, 3);
                lamps += 1;
            } else if trees < 12 && area_free(blocked, x, z, 2) {
                self.place_tree(x, h, z, seed);
                mark(blocked, x, z, 3);
                trees += 1;
            }
        }

        // arbustos sueltos
        for &(x, z) in &spots {
            if !blocked[col(x, z)] && hash(x, z, seed.wrapping_add(5)) < 0.07 {
                self.set(x, heights[col(x, z)] + 1, z, LEAVES);
            }
        }
    }

    // poste de madera con glowstone arriba
    fn place_lamp(&mut self, x: i32, h: i32, z: i32) {
        self.set(x, h + 1, z, WOOD);
        self.set(x, h + 2, z, GLOWSTONE);
    }

    fn place_tree(&mut self, x: i32, h: i32, z: i32, seed: u32) {
        let trunk = 3 + (hash(x, z, seed.wrapping_add(1)) * 3.0) as i32;
        for y in 1..=trunk {
            self.set(x, h + y, z, WOOD);
        }

        // hojas alrededor de la punta del tronco
        let top = h + trunk;
        for dy in -1..=1 {
            for dz in -2i32..=2 {
                for dx in -2i32..=2 {
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

    // los bloques emisivos se vuelven luces
    pub fn lights(&self) -> Vec<Light> {
        let mut list = Vec::new();
        for y in 0..H {
            for z in 0..D {
                for x in 0..W {
                    let b = self.get(x, y, z);
                    if b == GLOWSTONE {
                        list.push(Light {
                            pos: Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5),
                            cell: [x, y, z],
                            color: Vec3::new(1.0, 0.72, 0.4),
                            power: 2.0,
                        });
                    } else if b == LAVA && self.get(x, y + 1, z) == AIR {
                        // la luz de la lava va un poco arriba para que alumbre el suelo de al lado
                        list.push(Light {
                            pos: Vec3::new(x as f32 + 0.5, y as f32 + 1.3, z as f32 + 0.5),
                            cell: [x, y, z],
                            color: Vec3::new(1.0, 0.35, 0.1),
                            power: 2.5,
                        });
                    }
                }
            }
        }
        list
    }

    // oclusion ambiental estilo minecraft:
    // las esquinas donde se juntan bloques se ven mas oscuras
    pub fn ambient_occlusion(&self, hit: &Hit) -> f32 {
        let n = [hit.normal.x as i32, hit.normal.y as i32, hit.normal.z as i32];
        let c = hit.cell;

        // posicion dentro del bloque (0 a 1)
        let lx = hit.point.x - c[0] as f32;
        let ly = hit.point.y - c[1] as f32;
        let lz = hit.point.z - c[2] as f32;

        // los dos ejes que forman la cara
        let (a, b, fa, fb) = if n[1] != 0 {
            ([1, 0, 0], [0, 0, 1], lx, lz)
        } else if n[0] != 0 {
            ([0, 0, 1], [0, 1, 0], lz, ly)
        } else {
            ([1, 0, 0], [0, 1, 0], lx, ly)
        };

        // capa de bloques que esta enfrente de la cara
        let fx = c[0] + n[0];
        let fy = c[1] + n[1];
        let fz = c[2] + n[2];

        let solid = |da: i32, db: i32| -> f32 {
            let block = self.get(
                fx + a[0] * da + b[0] * db,
                fy + a[1] * da + b[1] * db,
                fz + a[2] * da + b[2] * db,
            );
            if blocks_ao(block) { 1.0 } else { 0.0 }
        };

        // como en minecraft: si los dos lados estan tapados la esquina queda negra
        let corner = |sa: i32, sb: i32| -> f32 {
            let s1 = solid(sa, 0);
            let s2 = solid(0, sb);
            let cc = solid(sa, sb);
            if s1 + s2 > 1.5 { 0.0 } else { (3.0 - s1 - s2 - cc) / 3.0 }
        };

        let c00 = corner(-1, -1);
        let c10 = corner(1, -1);
        let c01 = corner(-1, 1);
        let c11 = corner(1, 1);

        // mezclo las 4 esquinas segun donde cayo el punto
        let fa = fa.clamp(0.0, 1.0);
        let fb = fb.clamp(0.0, 1.0);
        let top = c00 + (c10 - c00) * fa;
        let bottom = c01 + (c11 - c01) * fa;
        top + (bottom - top) * fb
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
            // el agua, el vidrio, el hielo y lo emisivo dejan pasar la luz
            m.transparency == 0.0 && m.emission == 0.0
        });
        hit.is_some()
    }
}

// bloques que oscurecen las esquinas de sus vecinos
fn blocks_ao(b: u8) -> bool {
    b != AIR && b != WATER && b != GLASS && b != LAVA && b != GLOWSTONE
}

fn mark(blocked: &mut [bool], x: i32, z: i32, r: i32) {
    for dz in -r..=r {
        for dx in -r..=r {
            let cx = x + dx;
            let cz = z + dz;
            if cx >= 0 && cx < W && cz >= 0 && cz < D {
                blocked[col(cx, cz)] = true;
            }
        }
    }
}

fn area_free(blocked: &[bool], x: i32, z: i32, r: i32) -> bool {
    for dz in -r..=r {
        for dx in -r..=r {
            let cx = x + dx;
            let cz = z + dz;
            if cx >= 0 && cx < W && cz >= 0 && cz < D && blocked[col(cx, cz)] {
                return false;
            }
        }
    }
    true
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
