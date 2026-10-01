use crate::noise::{hash, noise};
use crate::vec3::Vec3;

// tamaño de las texturas de los bloques, estilo minecraft
pub const TEX_SIZE: usize = 16;

// indices de cada textura dentro de la lista (mismo orden que create_all)
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
pub const T_PLANKS: usize = 13;
pub const T_PLANKS_NORMAL: usize = 14;
pub const T_GLASS: usize = 15;
pub const T_LAVA: usize = 16;
pub const T_COBBLE: usize = 17;
pub const T_COBBLE_NORMAL: usize = 18;

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

    // u y v van de 0 a 1, agarro el pixel mas cercano (pixelado como minecraft)
    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let s = self.size as i32;
        let x = ((u * self.size as f32) as i32).clamp(0, s - 1);
        let y = ((v * self.size as f32) as i32).clamp(0, s - 1);
        self.pixels[(y * s + x) as usize]
    }
}

// los colores se guardan en espacio lineal para que la luz se sume bien
// (al final se vuelve a pasar a sRGB en el render)
pub fn rgb(r: u8, g: u8, b: u8) -> Vec3 {
    Vec3::new(to_linear(r), to_linear(g), to_linear(b))
}

fn to_linear(c: u8) -> f32 {
    (c as f32 / 255.0).powf(2.2)
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
        planks(),
        planks_normal(),
        glass(),
        lava(),
        cobble(),
        cobble_normal(),
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
    if n > 0.9 { base * 0.6 } else { base }
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
    Texture::new(TEX_SIZE, |x, y| rgb(128, 128, 128) * (0.55 + 0.6 * stone_height(x, y)))
}

fn stone_normal() -> Texture {
    normal_from_height(2.5, stone_height)
}

fn sand() -> Texture {
    Texture::new(TEX_SIZE, |x, y| rgb(219, 205, 160) * (0.85 + 0.25 * hash(x, y, 4)))
}

// olitas con senos, se repiten bien en los bordes de la textura
fn water_height(x: i32, y: i32) -> f32 {
    let k = std::f32::consts::PI * 2.0 / TEX_SIZE as f32;
    ((x as f32 * k * 2.0).sin() + ((x + 2 * y) as f32 * k).sin()) * 0.25 + 0.5
}

fn water() -> Texture {
    Texture::new(TEX_SIZE, |x, y| rgb(45, 95, 200) * (0.8 + 0.4 * water_height(x, y)))
}

fn water_normal() -> Texture {
    normal_from_height(1.5, water_height)
}

fn log_side() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        // rayas verticales de la corteza
        let stripe = hash(x, 0, 5);
        let n = hash(x, y, 6);
        rgb(105, 80, 50) * (0.55 + 0.4 * stripe + 0.15 * n)
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
        let c = rgb(58, 125, 42) * (0.6 + 0.6 * n);
        if n > 0.85 { c * 0.4 } else { c }
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

// tablas de 4 pixeles de alto con una rayita oscura entre cada una
fn planks_height(x: i32, y: i32) -> f32 {
    let row = y / 4;
    let shift = if row % 2 == 0 { 0 } else { 8 };
    if y % 4 == 3 || (x + shift) % 16 == 0 {
        0.0
    } else {
        0.7 + 0.3 * hash(x / 3, y, 30 + row as u32)
    }
}

fn planks() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        let grain = hash(x, y / 4, 31) * 0.1;
        rgb(170, 130, 80) * (0.45 + 0.55 * planks_height(x, y) + grain)
    })
}

fn planks_normal() -> Texture {
    normal_from_height(1.5, planks_height)
}

fn glass() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        let border = x == 0 || y == 0 || x == 15 || y == 15;
        // reflejito en diagonal como el vidrio de minecraft
        let shine = (x - y == 3 || x - y == 4) && x < 11;
        if border {
            rgb(185, 205, 215)
        } else if shine {
            rgb(245, 250, 255)
        } else {
            rgb(215, 232, 240)
        }
    })
}

fn lava() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        let n = noise(x as f32 * 0.3, y as f32 * 0.3, 40) * 0.75 + hash(x, y, 41) * 0.25;
        if n > 0.6 {
            rgb(255, 225, 110)
        } else if n > 0.42 {
            rgb(250, 130, 30)
        } else {
            rgb(175, 45, 12)
        }
    })
}

// piedras de 4x4 medio desordenadas, cada fila corrida un poco
fn cobble_height(x: i32, y: i32) -> f32 {
    let row = y / 4;
    let shift = (row % 2) * 2;
    let lx = (x + shift) % 4;
    let ly = y % 4;
    if lx == 0 || ly == 0 {
        return 0.1; // junta entre piedras
    }
    let stone = 0.55 + 0.45 * hash((x + shift) / 4, row, 50);
    let edge = if lx == 3 || ly == 3 { 0.15 } else { 0.0 };
    stone - edge
}

fn cobble() -> Texture {
    Texture::new(TEX_SIZE, |x, y| {
        let n = 0.9 + 0.2 * hash(x, y, 51);
        rgb(125, 125, 130) * ((0.35 + 0.7 * cobble_height(x, y)) * n)
    })
}

fn cobble_normal() -> Texture {
    normal_from_height(2.0, cobble_height)
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
