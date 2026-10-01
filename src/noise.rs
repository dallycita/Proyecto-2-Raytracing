// funciones de ruido hechas a mano (no se pueden usar librerias)

// numero "random" entre 0 y 1, siempre da lo mismo para la misma entrada
pub fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = seed.wrapping_mul(0x9E37_79B9)
        ^ (x as u32).wrapping_mul(0x85EB_CA6B)
        ^ (y as u32).wrapping_mul(0xC2B2_AE35);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0xFFFF) as f32 / 65535.0
}

// value noise: mezcla suave entre las 4 esquinas de la celda
pub fn noise(x: f32, y: f32, seed: u32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let ix = x0 as i32;
    let iy = y0 as i32;

    // suavizado para que no se vean las lineas de la cuadricula
    let fx = x - x0;
    let fy = y - y0;
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);

    let a = hash(ix, iy, seed);
    let b = hash(ix + 1, iy, seed);
    let c = hash(ix, iy + 1, seed);
    let d = hash(ix + 1, iy + 1, seed);

    let top = a + (b - a) * sx;
    let bottom = c + (d - c) * sx;
    top + (bottom - top) * sy
}

// varias capas de ruido juntas, da un terreno mas natural
pub fn fbm(x: f32, y: f32, seed: u32, octaves: u32) -> f32 {
    let mut total = 0.0;
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for i in 0..octaves {
        total += noise(x * freq, y * freq, seed.wrapping_add(i * 31)) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    total / norm
}

// generador random sencillo (xorshift), cada pixel usa el suyo
pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Rng {
        // revuelvo la semilla para que pixeles vecinos no salgan parecidos
        let mut rng = Rng {
            state: seed.wrapping_mul(0x9E37_79B9) ^ 0x6A09_E667 | 1,
        };
        rng.next();
        rng.next();
        rng
    }

    // numero entre 0 y 1
    pub fn next(&mut self) -> f32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        (x & 0xFFFF) as f32 / 65535.0
    }
}
