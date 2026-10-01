use crate::noise::{hash, Rng};
use crate::scene::Scene;
use crate::vec3::Vec3;
use crate::world::{Hit, AIR, LAVA, WATER};

// cuantos rebotes maximo (reflejos y refracciones)
pub const MAX_DEPTH: u32 = 4;
const EPS: f32 = 0.001;
// las luces que estan mas lejos que esto ni se calculan
const LIGHT_RANGE: f32 = 10.0;

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

// vector random chiquito, sirve para las sombras suaves
fn random_offset(rng: &mut Rng, radius: f32) -> Vec3 {
    Vec3::new(
        (rng.next() - 0.5) * 2.0 * radius,
        (rng.next() - 0.5) * 2.0 * radius,
        (rng.next() - 0.5) * 2.0 * radius,
    )
}

// medium = el bloque transparente por donde va el rayo (AIR si va por el aire)
pub fn trace(scene: &Scene, origin: Vec3, dir: Vec3, depth: u32, medium: u8, rng: &mut Rng) -> Vec3 {
    let hit = scene.world.march(origin, dir, 200.0, |b| b != medium);

    let (color, dist) = match hit {
        Some(h) => (shade(scene, &h, dir, depth, medium, rng), h.t),
        None => (scene.sky(dir), 6.0),
    };

    if medium == AIR {
        return color;
    }

    // dentro del agua el color se va tiñendo con la distancia (el vidrio casi nada)
    let m = &scene.materials[medium as usize];
    let fog = scene.textures[m.tex_side].sample(0.5, 0.5) * 0.4;
    let density = if medium == WATER { 0.35 } else { 0.05 };
    let f = (-dist * density).exp();
    color * f + fog * (1.0 - f)
}

fn shade(scene: &Scene, hit: &Hit, dir: Vec3, depth: u32, medium: u8, rng: &mut Rng) -> Vec3 {
    // el rayo va saliendo del agua (o vidrio/hielo) hacia el aire
    if hit.block == AIR {
        if depth >= MAX_DEPTH {
            return scene.sky(dir);
        }
        let m = &scene.materials[medium as usize];
        return match refract(dir, hit.normal, m.ior) {
            Some(r) => trace(scene, hit.point - hit.normal * EPS, r, depth + 1, AIR, rng),
            None => trace(scene, hit.point + hit.normal * EPS, dir.reflect(hit.normal), depth + 1, medium, rng),
        };
    }

    let m = &scene.materials[hit.block as usize];
    let (mut u, mut v, tangent, bitangent) = face_uv(hit);
    // el agua y la lava se mueven con el tiempo
    if hit.block == WATER {
        u = (u + scene.time * 0.15).fract();
    }
    if hit.block == LAVA {
        v = (v + scene.time * 0.04).fract();
    }

    let tex = if hit.normal.y > 0.5 { m.tex_top } else { m.tex_side };
    let mut base = scene.textures[tex].sample(u, v);

    // cada bloque con un tono un poquito distinto para que no se vea tan repetido
    if m.transparency == 0.0 && m.emission == 0.0 {
        let c = hit.cell;
        base = base * (0.92 + 0.16 * hash(c[0] * 7 + c[1] * 13, c[2], 99));
    }

    // mapa normal: inclina la normal segun la textura
    let mut normal = hit.normal;
    if let Some(nm) = m.normal_map {
        let t = scene.textures[nm].sample(u, v);
        normal = (tangent * t.x + bitangent * t.y + hit.normal * t.z).normalize();
    }

    let light = &scene.light;
    let view = -dir;
    // punto un poquito afuera para que el rayo no choque consigo mismo
    let start = hit.point + hit.normal * EPS;

    // luz ambiente: viene mas del cielo que del suelo, y las esquinas se oscurecen
    let ao = scene.world.ambient_occlusion(hit);
    let ambient = light.ground_ambient.lerp(light.sky_ambient, normal.y * 0.5 + 0.5) * (0.25 + 0.75 * ao);
    let mut diffuse = base.mult(ambient);
    let mut specular = Vec3::zero();

    // sol (o luna). la direccion se mueve un poquito en cada muestra y asi la sombra queda suave
    let sun = (light.dir + random_offset(rng, 0.03)).normalize();
    let ndl = normal.dot(sun);
    if ndl > 0.0 && hit.normal.dot(sun) > 0.0 && !scene.world.in_shadow(start, sun, 100.0, &scene.materials) {
        diffuse += base.mult(light.color) * ndl;
        specular += light.color * phong(sun, normal, view, m.specular);
    }

    // luces de glowstone y lava
    for l in &scene.lights {
        // el bloque que da la luz no se alumbra a si mismo
        if l.cell == hit.cell {
            continue;
        }
        if (l.pos - hit.point).length() > LIGHT_RANGE {
            continue;
        }
        let target = l.pos + random_offset(rng, 0.3);
        let to_light = target - hit.point;
        let dist = to_light.length();
        if dist < 0.01 {
            continue;
        }
        let l_dir = to_light * (1.0 / dist);
        let ndl = normal.dot(l_dir);
        if ndl <= 0.0 || hit.normal.dot(l_dir) <= 0.0 {
            continue;
        }
        if scene.world.in_shadow(start, l_dir, dist, &scene.materials) {
            continue;
        }
        // se apaga suave hasta llegar al limite
        let fade = (1.0 - dist / LIGHT_RANGE).max(0.0);
        let power = l.power * light.lamp_scale * fade * fade / (1.0 + dist * dist * 0.3);
        diffuse += base.mult(l.color) * (ndl * power);
        specular += l.color * (phong(l_dir, normal, view, m.specular) * power);
    }

    let local = diffuse * m.albedo[0] + specular * m.albedo[1];

    // fresnel: viendo de lado el agua/vidrio refleja mas y deja pasar menos
    let cos = view.dot(normal).clamp(0.0, 1.0);
    let fresnel = (1.0 - cos).powi(5) * 0.8;
    let kr = m.reflectivity + m.transparency * fresnel;
    let kt = m.transparency * (1.0 - fresnel);

    let mut color = local * (1.0 - m.reflectivity - m.transparency);

    // reflexion
    if depth < MAX_DEPTH && kr > 0.0 {
        let mut r = dir.reflect(normal);
        if r.dot(hit.normal) < 0.0 {
            // si el mapa normal lo manda para adentro uso la normal de la cara
            r = dir.reflect(hit.normal);
        }
        color += trace(scene, start, r, depth + 1, medium, rng) * kr;
    }

    // refraccion
    if depth < MAX_DEPTH && kt > 0.0 {
        let eta = scene.ior_of(medium) / m.ior;
        let refracted = match refract(dir, normal, eta) {
            Some(r) if r.dot(hit.normal) < 0.0 => Some(r),
            _ => refract(dir, hit.normal, eta),
        };
        let behind = match refracted {
            // ahora el rayo va por dentro de este bloque
            Some(r) => trace(scene, hit.point - hit.normal * EPS, r, depth + 1, hit.block, rng),
            None => trace(scene, start, dir.reflect(hit.normal), depth + 1, medium, rng),
        };
        color += behind * kt;
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
