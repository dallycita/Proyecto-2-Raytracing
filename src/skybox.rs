use crate::noise::{fbm, hash};
use crate::texture::{rgb, Texture};
use crate::vec3::Vec3;

// momentos del dia
pub const DAY: u8 = 0;
pub const SUNSET: u8 = 1;
pub const NIGHT: u8 = 2;

// resolucion de cada cara del cubo del cielo
const SKY_SIZE: usize = 128;

// la luz sale de la misma direccion donde se pinta el sol (o la luna)
pub fn light_dir(mode: u8) -> Vec3 {
    match mode {
        DAY => Vec3::new(0.6, 0.55, 0.45).normalize(),
        SUNSET => Vec3::new(0.85, 0.13, 0.3).normalize(),
        _ => Vec3::new(-0.5, 0.6, -0.4).normalize(),
    }
}

// colores de cada cielo
struct Palette {
    horizon: Vec3,     // horizonte del lado del sol
    horizon_far: Vec3, // horizonte del otro lado
    zenith: Vec3,      // arriba
    ground: Vec3,      // abajo del horizonte
    cloud: Vec3,
    cloud_strength: f32,
    disk: Vec3, // sol o luna
    glow: Vec3, // brillo alrededor del sol
    stars: bool,
}

fn palette(mode: u8) -> Palette {
    match mode {
        DAY => Palette {
            horizon: rgb(180, 210, 238),
            horizon_far: rgb(160, 195, 235),
            zenith: rgb(40, 100, 200),
            ground: rgb(95, 105, 115),
            cloud: rgb(255, 255, 255),
            cloud_strength: 0.85,
            disk: rgb(255, 248, 230) * 12.0,
            glow: rgb(255, 225, 180) * 0.5,
            stars: false,
        },
        SUNSET => Palette {
            horizon: rgb(255, 140, 70),
            horizon_far: rgb(150, 105, 150),
            zenith: rgb(45, 55, 120),
            ground: rgb(60, 45, 55),
            cloud: rgb(255, 170, 140),
            cloud_strength: 0.8,
            disk: rgb(255, 190, 120) * 10.0,
            glow: rgb(255, 130, 50),
            stars: false,
        },
        _ => Palette {
            horizon: rgb(25, 32, 65),
            horizon_far: rgb(18, 22, 48),
            zenith: rgb(3, 4, 14),
            ground: rgb(8, 8, 14),
            cloud: rgb(28, 32, 50),
            cloud_strength: 0.6,
            disk: rgb(225, 232, 255) * 4.0,
            glow: rgb(120, 140, 210) * 0.2,
            stars: true,
        },
    }
}

// skybox de 6 caras: +x, -x, +y, -y, +z, -z
pub struct Skybox {
    faces: Vec<Texture>,
}

impl Skybox {
    pub fn new(mode: u8) -> Skybox {
        let p = palette(mode);
        let sun = light_dir(mode);
        let mut faces = Vec::new();
        for face in 0..6 {
            // pinto cada pixel de la cara viendo hacia que direccion queda
            let tex = Texture::new(SKY_SIZE, |i, j| {
                let u = (i as f32 + 0.5) / SKY_SIZE as f32 * 2.0 - 1.0;
                let v = (j as f32 + 0.5) / SKY_SIZE as f32 * 2.0 - 1.0;
                let dir = face_direction(face, u, v).normalize();
                sky_color(&p, sun, dir, face as i32, i, j)
            });
            faces.push(tex);
        }
        Skybox { faces }
    }

    // busca en que cara cae la direccion y lee esa textura
    pub fn sample(&self, d: Vec3) -> Vec3 {
        let ax = d.x.abs();
        let ay = d.y.abs();
        let az = d.z.abs();

        let (face, u, v) = if ax >= ay && ax >= az {
            if d.x > 0.0 { (0, -d.z / ax, -d.y / ax) } else { (1, d.z / ax, -d.y / ax) }
        } else if ay >= az {
            if d.y > 0.0 { (2, d.x / ay, d.z / ay) } else { (3, d.x / ay, -d.z / ay) }
        } else if d.z > 0.0 {
            (4, d.x / az, -d.y / az)
        } else {
            (5, -d.x / az, -d.y / az)
        };

        self.faces[face].sample((u + 1.0) * 0.5, (v + 1.0) * 0.5)
    }
}

// lo contrario de sample: de (cara, u, v) a direccion
fn face_direction(face: usize, u: f32, v: f32) -> Vec3 {
    match face {
        0 => Vec3::new(1.0, -v, -u),
        1 => Vec3::new(-1.0, -v, u),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, -v),
        4 => Vec3::new(u, -v, 1.0),
        _ => Vec3::new(-u, -v, -1.0),
    }
}

// cuanto hay de nube en esa direccion (0 a 1)
fn cloud_amount(dir: Vec3) -> f32 {
    if dir.y < 0.03 {
        return 0.0;
    }
    // proyecto la direccion a un "techo" plano
    let px = dir.x / dir.y * 2.0;
    let pz = dir.z / dir.y * 2.0;
    let n = fbm(px, pz, 77, 4);
    let cloud = ((n - 0.5) / 0.2).clamp(0.0, 1.0);
    // se desvanecen cerca del horizonte
    cloud * (dir.y * 5.0).min(1.0)
}

fn sky_color(p: &Palette, sun: Vec3, dir: Vec3, face: i32, i: i32, j: i32) -> Vec3 {
    // el horizonte cambia de color si esta del lado del sol o del otro
    let flat = Vec3::new(dir.x, 0.0, dir.z).normalize();
    let sun_flat = Vec3::new(sun.x, 0.0, sun.z).normalize();
    let side = flat.dot(sun_flat) * 0.5 + 0.5;
    let horizon = p.horizon_far.lerp(p.horizon, side);

    let mut c = if dir.y >= 0.0 {
        horizon.lerp(p.zenith, dir.y.powf(0.45))
    } else {
        horizon.lerp(p.ground, (-dir.y).sqrt())
    };

    // estrellas random
    if p.stars && dir.y > 0.05 && hash(i + face * 1000, j, 21) > 0.993 {
        c = Vec3::new(1.0, 1.0, 1.0) * (0.3 + 0.9 * hash(i, j + face * 1000, 22));
    }

    // brillo alrededor del sol
    let s = dir.dot(sun).max(0.0);
    c += p.glow * (s.powf(8.0) * 0.3 + s.powf(64.0));

    c = c.lerp(p.cloud, cloud_amount(dir) * p.cloud_strength);

    if s > 0.9985 {
        c = p.disk;
    }
    c
}
