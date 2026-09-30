use std::sync::Mutex;
use std::thread;

use crate::camera::OrbitCamera;
use crate::raytracer::trace;
use crate::scene::Scene;
use crate::world::AIR;

// cada trabajo son unas cuantas filas de la imagen
const ROWS_PER_JOB: usize = 4;

// renderiza con varios hilos (solo std, sin librerias)
// los hilos van agarrando grupos de filas de una lista compartida,
// asi si una parte es mas pesada (agua, reflejos) los demas no se quedan esperando
pub fn render(frame: &mut [u8], width: usize, height: usize, scene: &Scene, cam: &OrbitCamera) {
    let threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);

    // esto se calcula una vez por frame y no por pixel
    let (forward, right, up) = cam.basis();
    let eye = cam.eye();
    let aspect = width as f32 / height as f32;
    let fov_scale = (cam.fov * 0.5).tan();

    let jobs: Vec<(usize, &mut [u8])> = frame.chunks_mut(width * 4 * ROWS_PER_JOB).enumerate().collect();
    let queue = Mutex::new(jobs);

    thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let job = queue.lock().unwrap().pop();
                let Some((index, chunk)) = job else { break };

                let first_row = index * ROWS_PER_JOB;
                let rows = chunk.len() / (width * 4);
                for r in 0..rows {
                    let y = first_row + r;
                    for x in 0..width {
                        // de pixel a direccion del rayo
                        let sx = (2.0 * (x as f32 + 0.5) / width as f32 - 1.0) * aspect * fov_scale;
                        let sy = (1.0 - 2.0 * (y as f32 + 0.5) / height as f32) * fov_scale;
                        let dir = (forward + right * sx + up * sy).normalize();

                        let c = trace(scene, eye, dir, 0, AIR);

                        let i = (r * width + x) * 4;
                        chunk[i] = to_byte(c.x);
                        chunk[i + 1] = to_byte(c.y);
                        chunk[i + 2] = to_byte(c.z);
                        chunk[i + 3] = 255;
                    }
                }
            });
        }
    });
}

fn to_byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0) as u8
}
