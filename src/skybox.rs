use crate::noise::{fbm, hash};
use crate::texture::{rgb, Texture};
use crate::vec3::Vec3;

// resolucion de cada cara del cubo del cielo
const SKY_SIZE: usize = 128;

// la luz del sol sale de la misma direccion donde se pinta el sol en el cielo
pub fn sun_dir() -> Vec3 {
    Vec3::new(0.6, 0.55, 0.45).normalize()
}

pub fn moon_dir() -> Vec3 {
    Vec3::new(-0.5, 0.6, -0.4).normalize()
}

// skybox de 6 caras: +x, -x, +y, -y, +z, -z
pub struct Skybox {
    faces: Vec<Texture>,
}

impl Skybox {
    pub fn new(night: bool) -> Skybox {
        let mut faces = Vec::new();
        for face in 0..6 {
            // pinto cada pixel de la cara viendo hacia que direccion queda
            let tex = Texture::new(SKY_SIZE, |i, j| {
                let u = (i as f32 + 0.5) / SKY_SIZE as f32 * 2.0 - 1.0;
                let v = (j as f32 + 0.5) / SKY_SIZE as f32 * 2.0 - 1.0;
                let dir = face_direction(face, u, v).normalize();
                if night {
                    night_color(dir, face as i32, i, j)
                } else {
                    day_color(dir)
                }
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

fn day_color(dir: Vec3) -> Vec3 {
    let horizon = rgb(185, 215, 240);
    let zenith = rgb(60, 120, 205);

    let mut c = if dir.y >= 0.0 {
        horizon.lerp(zenith, dir.y.sqrt())
    } else {
        horizon.lerp(rgb(110, 125, 140), (-dir.y).sqrt())
    };

    c = c.lerp(rgb(250, 250, 255), cloud_amount(dir) * 0.85);

    // sol con un brillito alrededor
    let s = dir.dot(sun_dir()).max(0.0);
    if s > 0.9985 {
        c = rgb(255, 250, 225) * 1.6;
    } else {
        c += rgb(255, 220, 160) * (s.powf(80.0) * 0.5);
    }
    c
}

fn night_color(dir: Vec3, face: i32, i: i32, j: i32) -> Vec3 {
    let horizon = rgb(25, 30, 60);
    let zenith = rgb(4, 6, 18);

    let mut c = if dir.y >= 0.0 {
        horizon.lerp(zenith, dir.y.sqrt())
    } else {
        horizon * 0.6
    };

    // estrellas random
    if dir.y > 0.05 && hash(i + face * 1000, j, 21) > 0.993 {
        c = rgb(255, 255, 255) * (0.5 + 0.5 * hash(i, j + face * 1000, 22));
    }

    c = c.lerp(rgb(40, 45, 70), cloud_amount(dir) * 0.5);

    // luna
    let s = dir.dot(moon_dir()).max(0.0);
    if s > 0.998 {
        c = rgb(230, 235, 255) * 1.2;
    } else {
        c += rgb(120, 140, 200) * (s.powf(40.0) * 0.2);
    }
    c
}
