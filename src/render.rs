use std::sync::Mutex;
use std::thread;

use crate::camera::OrbitCamera;
use crate::noise::Rng;
use crate::raytracer::trace;
use crate::scene::Scene;
use crate::vec3::Vec3;
use crate::world::AIR;

// cada trabajo son unas cuantas filas de la imagen
const ROWS_PER_JOB: usize = 4;
// cuando la camara esta quieta se van promediando muestras hasta este limite
const MAX_SAMPLES: u32 = 32;

pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pub samples: u32,
    accum: Vec<Vec3>, // promedio de las muestras de cada pixel
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Renderer {
        Renderer {
            width,
            height,
            samples: 0,
            accum: vec![Vec3::zero(); width * height],
        }
    }

    // se llama cuando algo cambia (camara, terreno, hora del dia)
    pub fn reset(&mut self) {
        self.samples = 0;
    }

    // renderiza una muestra mas con varios hilos (solo std, sin librerias)
    // los hilos van agarrando grupos de filas de una cola compartida,
    // asi si una parte pesa mas (agua, vidrio) los demas no se quedan esperando
    pub fn render(&mut self, frame: &mut [u8], scene: &Scene, cam: &OrbitCamera) {
        self.samples += 1;
        let sample = self.samples;
        // la primera muestra reemplaza, las demas se van promediando
        let weight = 1.0 / sample.min(MAX_SAMPLES) as f32;

        let width = self.width;
        let height = self.height;
        let threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);

        // esto se calcula una vez por frame y no por pixel
        let (forward, right, up) = cam.basis();
        let eye = cam.eye();
        let aspect = width as f32 / height as f32;
        let fov_scale = (cam.fov * 0.5).tan();
        let exposure = scene.light.exposure;

        let jobs: Vec<(usize, &mut [Vec3], &mut [u8])> = self
            .accum
            .chunks_mut(width * ROWS_PER_JOB)
            .zip(frame.chunks_mut(width * 4 * ROWS_PER_JOB))
            .enumerate()
            .map(|(i, (acc, out))| (i, acc, out))
            .collect();
        let queue = Mutex::new(jobs);

        thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| loop {
                    let job = queue.lock().unwrap().pop();
                    let Some((index, acc, out)) = job else { break };

                    let first_row = index * ROWS_PER_JOB;
                    let rows = acc.len() / width;
                    for r in 0..rows {
                        let y = first_row + r;
                        for x in 0..width {
                            let seed = ((y * width + x) as u32).wrapping_mul(747_796_405) ^ sample.wrapping_mul(2_891_336_453);
                            let mut rng = Rng::new(seed);

                            // primera muestra al centro del pixel, las demas movidas un poquito (antialiasing)
                            let (jx, jy) = if sample == 1 { (0.5, 0.5) } else { (rng.next(), rng.next()) };

                            // de pixel a direccion del rayo
                            let sx = (2.0 * (x as f32 + jx) / width as f32 - 1.0) * aspect * fov_scale;
                            let sy = (1.0 - 2.0 * (y as f32 + jy) / height as f32) * fov_scale;
                            let dir = (forward + right * sx + up * sy).normalize();

                            let c = trace(scene, eye, dir, 0, AIR, &mut rng);

                            let i = r * width + x;
                            acc[i] = acc[i] * (1.0 - weight) + c * weight;

                            let final_color = acc[i] * exposure;
                            let o = i * 4;
                            out[o] = to_byte(final_color.x);
                            out[o + 1] = to_byte(final_color.y);
                            out[o + 2] = to_byte(final_color.z);
                            out[o + 3] = 255;
                        }
                    }
                });
            }
        });
    }
}

// tone mapping ACES (para que lo muy brillante no se queme) y luego gamma
fn to_byte(v: f32) -> u8 {
    let x = v.max(0.0);
    let mapped = ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0);
    (mapped.powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}
