use crate::noise::{hash, noise};
use crate::vec3::Vec3;

// tamaño de las texturas de los bloques, estilo minecraft
pub const TEX_SIZE: usize = 16;

// indices de cada textura dentro de la lista (tienen que ir en el mismo orden que create_all)
pub const T_GRASS_TOP: usize = 0;
pub const T_GRASS_SIDE: usize = 1;
pub const T_DIRT: usize = 2;
pub const T_STONE: usize = 3;
pub const T_STONE_NORMAL: usize = 4;
pub const T_SAND: usize = 5;
pub const T_WATER: usize = 6;
pub const T_WATER_NORMAL: usize = 7;
pub const T_LOG_SIDE: usize = 8;
pub const T_LOG_TOP: usize = 9;
pub const T_LEAVES: usize = 10;
pub const T_ICE: usize = 11;
pub const T_GLOWSTONE: usize = 12;

pub struct Texture {
    pub size: usize,
    pub pixels: Vec<Vec3>,
}

impl Texture {
    // crea la textura pintando cada pixel con la funcion que le pasemos
    pub fn new<F: Fn(i32, i32) -> Vec3>(size: usize, f: F) -> Texture {
        let mut pixels = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                pixels.push(f(x as i32, y as i32));
            }
        }
        Texture { size, pixels }
    }

    // u y v van de 0 a 1, agarro el pixel mas cercano (se ve pixelado como minecraft)
    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let s = self.size as i32;
        let x = ((u * self.size as f32) as i32).clamp(0, s - 1);
        let y = ((v * self.size as f32) as i32).clamp(0, s - 1);
        self.pixels[(y * s + x) as usize]
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> Vec3 {
    Vec3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

pub fn create_all() -> Vec<Texture> {
    vec![
        grass_top(),
        grass_side(),
        dirt(),
        stone(),
        stone_normal(),
        sand(),
        water(),
        water_normal(),
        log_side(),
        log_top(),
        leaves(),
        ice(),
        glowstone(),
    ]
}

// ---------- colores base ----------

fn grass_color(x: i32, y: i32) -> Vec3 {
    rgb(96, 158, 58) * (0.8 + 0.35 * hash(x, y, 1))
}

fn dirt_color(x: i32, y: i32) -> Vec3 {
    let n = hash(x, y, 2);
    let base = rgb(134, 96, 67) * (0.8 + 0.3 * n);
    // piedritas mas oscuras
    if n > 0.9 { base * 0.7 } else { base }
}

// ---------- texturas ----------

fn grass_top() -> Texture {
    Texture::new(TEX_SIZE, grass_color)
}

fn grass_side() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        // el pasto cae un poquito por el borde de arriba
        let edge = 3 + (hash(x, 0, 3) * 2.0) as i32;
        if y < edge { grass_color(x, y) } else { dirt_color(x, y) }
    })
}

fn dirt() -> Texture {
    Texture::new(TEX_SIZE, dirt_color)
}

// la misma altura se usa para el color y para el mapa normal
fn stone_height(x: i32, y: i32) -> f32 {
    noise(x as f32 * 0.4, y as f32 * 0.4, 11) * 0.7 + hash(x, y, 12) * 0.3
}

fn stone() -> Texture {
    Texture::new(TEX_SIZE, |x, y| rgb(128, 128, 128) * (0.65 + 0.5 * stone_height(x, y)))
}

fn stone_normal() -> Texture {
    normal_from_height(2.5, stone_height)
}

fn sand() -> Texture {
    Texture::new(TEX_SIZE, |x, y| rgb(219, 205, 160) * (0.88 + 0.2 * hash(x, y, 4)))
}

// olitas con senos, se repiten bien en los bordes de la textura
fn water_height(x: i32, y: i32) -> f32 {
    let k = std::f32::consts::PI * 2.0 / TEX_SIZE as f32;
    ((x as f32 * k * 2.0).sin() + ((x + 2 * y) as f32 * k).sin()) * 0.25 + 0.5
}

fn water() -> Texture {
    Texture::new(TEX_SIZE, |x, y| rgb(45, 95, 200) * (0.85 + 0.3 * water_height(x, y)))
}

fn water_normal() -> Texture {
    normal_from_height(1.5, water_height)
}

fn log_side() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        // rayas verticales de la corteza
        let stripe = hash(x, 0, 5);
        let n = hash(x, y, 6);
        rgb(105, 80, 50) * (0.65 + 0.3 * stripe + 0.1 * n)
    })
}

fn log_top() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        // anillos del tronco
        let dx = x as f32 - 7.5;
        let dy = y as f32 - 7.5;
        let r = (dx * dx + dy * dy).sqrt();
        if r > 6.5 {
            rgb(105, 80, 50) * 0.9
        } else if (r * 1.3) as i32 % 2 == 0 {
            rgb(180, 145, 95)
        } else {
            rgb(150, 115, 72)
        }
    })
}

fn leaves() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        let n = hash(x, y, 8);
        let c = rgb(58, 125, 42) * (0.6 + 0.5 * n);
        if n > 0.85 { c * 0.5 } else { c }
    })
}

fn ice() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        // grietas blancas en diagonal
        let crack = (x + y * 2) % 7 == 0 && hash(x, y, 10) > 0.3;
        if crack {
            rgb(235, 245, 255)
        } else {
            rgb(150, 195, 245) * (0.9 + 0.15 * hash(x, y, 9))
        }
    })
}

fn glowstone() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        // manchas de 2x2 pixeles
        let n = hash(x / 2, y / 2, 13);
        if n > 0.66 {
            rgb(255, 235, 150)
        } else if n > 0.33 {
            rgb(245, 190, 90)
        } else {
            rgb(190, 130, 50)
        }
    })
}

// saca un mapa normal a partir de una funcion de altura
// si la altura sube hacia un lado la normal se inclina hacia el otro
fn normal_from_height<F: Fn(i32, i32) -> f32>(strength: f32, h: F) -> Texture {
    let s = TEX_SIZE as i32;
    Texture::new(TEX_SIZE, |x, y| {
        let left = h((x + s - 1) % s, y);
        let right = h((x + 1) % s, y);
        let up = h(x, (y + s - 1) % s);
        let down = h(x, (y + 1) % s);
        Vec3::new((left - right) * strength, (up - down) * strength, 1.0).normalize()
    })
}
