use crate::scene::Scene;
use crate::vec3::Vec3;
use crate::world::{Hit, AIR, WATER};

// cuantos rebotes maximo (reflejos y refracciones)
pub const MAX_DEPTH: u32 = 3;
const EPS: f32 = 0.001;
const LAMP_RANGE: f32 = 9.0;
const LAMP_COLOR: Vec3 = Vec3::new(1.0, 0.72, 0.4);

// ley de snell. n apunta hacia el lado de donde viene el rayo
// devuelve None si hay reflexion interna total
pub fn refract(d: Vec3, n: Vec3, eta: f32) -> Option<Vec3> {
    let cos_i = -d.dot(n);
    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
    if k < 0.0 {
        None
    } else {
        Some((d * eta + n * (eta * cos_i - k.sqrt())).normalize())
    }
}

// medium = el bloque transparente en el que va el rayo (AIR si va por el aire)
pub fn trace(scene: &Scene, origin: Vec3, dir: Vec3, depth: u32, medium: u8) -> Vec3 {
    let hit = scene.world.march(origin, dir, 200.0, |b| b != medium);

    let (color, dist) = match hit {
        Some(h) => (shade(scene, &h, dir, depth, medium), h.t),
        None => (scene.sky(dir), 6.0),
    };

    if medium == AIR {
        return color;
    }

    // dentro del agua/hielo el color se va tiñendo con la distancia
    let m = &scene.materials[medium as usize];
    let fog = scene.textures[m.tex_side].sample(0.5, 0.5) * 0.5;
    let f = (-dist * 0.3).exp();
    color * f + fog * (1.0 - f)
}

fn shade(scene: &Scene, hit: &Hit, dir: Vec3, depth: u32, medium: u8) -> Vec3 {
    // el rayo va saliendo del agua (o hielo) hacia el aire
    if hit.block == AIR {
        if depth >= MAX_DEPTH {
            return scene.sky(dir);
        }
        let m = &scene.materials[medium as usize];
        return match refract(dir, hit.normal, m.ior) {
            Some(r) => trace(scene, hit.point - hit.normal * EPS, r, depth + 1, AIR),
            None => trace(scene, hit.point + hit.normal * EPS, dir.reflect(hit.normal), depth + 1, medium),
        };
    }

    let m = &scene.materials[hit.block as usize];
    let (mut u, v, tangent, bitangent) = face_uv(hit);
    if hit.block == WATER {
        // muevo la textura del agua con el tiempo para que parezca que fluye
        u = (u + scene.time * 0.15).fract();
    }

    let tex = if hit.normal.y > 0.5 { m.tex_top } else { m.tex_side };
    let base = scene.textures[tex].sample(u, v);

    // mapa normal: inclina la normal segun la textura
    let mut normal = hit.normal;
    if let Some(nm) = m.normal_map {
        let t = scene.textures[nm].sample(u, v);
        normal = (tangent * t.x + bitangent * t.y + hit.normal * t.z).normalize();
    }

    let (light_dir, light_color, ambient) = scene.sun();
    let view = -dir;
    // punto un poquito afuera para que el rayo no choque consigo mismo
    let start = hit.point + hit.normal * EPS;

    let mut diffuse = base.mult(ambient);
    let mut specular = Vec3::zero();

    // sol (o luna) con sombra
    let ndl = normal.dot(light_dir);
    if ndl > 0.0 && !scene.world.in_shadow(start, light_dir, 100.0, &scene.materials) {
        diffuse += base.mult(light_color) * ndl;
        specular += light_color * phong(light_dir, normal, view, m.specular);
    }

    // lamparas de glowstone, solo las que estan cerca
    let lamp_power = scene.lamp_power();
    for lamp in &scene.lights {
        let to_lamp = *lamp - hit.point;
        let dist = to_lamp.length();
        if dist > LAMP_RANGE || dist < 0.9 {
            continue;
        }
        let l = to_lamp * (1.0 / dist);
        let ndl = normal.dot(l);
        if ndl <= 0.0 {
            continue;
        }
        if scene.world.in_shadow(start, l, dist, &scene.materials) {
            continue;
        }
        let power = lamp_power / (1.0 + dist * dist * 0.25);
        diffuse += base.mult(LAMP_COLOR) * (ndl * power);
        specular += LAMP_COLOR * (phong(l, normal, view, m.specular) * power);
    }

    let local = diffuse * m.albedo[0] + specular * m.albedo[1];
    let mut color = local * (1.0 - m.reflectivity - m.transparency);

    // reflexion
    if depth < MAX_DEPTH && m.reflectivity > 0.0 {
        let mut r = dir.reflect(normal);
        if r.dot(hit.normal) < 0.0 {
            // si la normal del mapa lo mando para adentro uso la normal normal
            r = dir.reflect(hit.normal);
        }
        color += trace(scene, start, r, depth + 1, medium) * m.reflectivity;
    }

    // refraccion
    if depth < MAX_DEPTH && m.transparency > 0.0 {
        let eta = scene.ior_of(medium) / m.ior;
        let refracted = match refract(dir, normal, eta) {
            Some(r) if r.dot(hit.normal) < 0.0 => Some(r),
            _ => refract(dir, hit.normal, eta),
        };
        let behind = match refracted {
            // ahora el rayo va por dentro de este bloque
            Some(r) => trace(scene, hit.point - hit.normal * EPS, r, depth + 1, hit.block),
            None => trace(scene, start, dir.reflect(hit.normal), depth + 1, medium),
        };
        color += behind * m.transparency;
    }

    // los emisivos suman su propia luz
    color + base * m.emission
}

// brillo especular de phong
fn phong(l: Vec3, n: Vec3, view: Vec3, shininess: f32) -> f32 {
    let r = (-l).reflect(n);
    r.dot(view).max(0.0).powf(shininess)
}

// coordenadas de textura segun la cara del cubo que se golpeo
// tambien devuelve hacia donde crecen u y v (para el mapa normal)
fn face_uv(hit: &Hit) -> (f32, f32, Vec3, Vec3) {
    let lx = (hit.point.x - hit.cell[0] as f32).clamp(0.0, 1.0);
    let ly = (hit.point.y - hit.cell[1] as f32).clamp(0.0, 1.0);
    let lz = (hit.point.z - hit.cell[2] as f32).clamp(0.0, 1.0);

    if hit.normal.y.abs() > 0.5 {
        // arriba / abajo
        (lx, lz, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0))
    } else if hit.normal.x.abs() > 0.5 {
        (lz, 1.0 - ly, Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, -1.0, 0.0))
    } else {
        (lx, 1.0 - ly, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0))
    }
}
