mod camera;
mod material;
mod noise;
mod raytracer;
mod render;
mod scene;
mod skybox;
mod texture;
mod vec3;
mod world;

use raylib::prelude::*;

use camera::OrbitCamera;
use render::Renderer;
use scene::Scene;

const WIN_W: i32 = 960;
const WIN_H: i32 = 540;

// modo video: 360 frames (12 segundos a 30 fps) con varias muestras cada uno
const VIDEO_FRAMES: u32 = 360;
const VIDEO_SAMPLES: u32 = 6;

const MODE_NAMES: [&str; 3] = ["dia", "atardecer", "noche"];

// lo que se dibuja en pantalla, se vuelve a crear al cambiar la calidad
struct Screen {
    frame: Vec<u8>,
    renderer: Renderer,
    texture: Texture2D,
}

fn make_screen(rl: &mut RaylibHandle, thread: &RaylibThread, scale: i32) -> Screen {
    // la imagen se calcula mas pequeña y luego se estira a la ventana
    let w = (WIN_W / scale) as usize;
    let h = (WIN_H / scale) as usize;
    let image = Image::gen_image_color(w as i32, h as i32, Color::BLACK);
    let texture = rl
        .load_texture_from_image(thread, &image)
        .expect("no se pudo crear la textura");
    Screen {
        frame: vec![0u8; w * h * 4],
        renderer: Renderer::new(w, h),
        texture,
    }
}

// guarda un frame en formato PPM (es solo texto + bytes, no necesita librerias)
fn save_ppm(index: u32, width: usize, height: usize, rgba: &[u8]) {
    let mut data = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    for px in rgba.chunks(4) {
        data.push(px[0]);
        data.push(px[1]);
        data.push(px[2]);
    }
    let path = format!("frames/frame_{:04}.ppm", index);
    if let Err(e) = std::fs::write(&path, data) {
        println!("no se pudo guardar {}: {}", path, e);
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIN_W, WIN_H)
        .title("Diorama - Raytracing")
        .build();
    rl.set_target_fps(60);

    let mut seed: u32 = 2024;
    let mut scene = Scene::new(seed);
    let mut cam = OrbitCamera::new(scene.center());

    println!("Materiales cargados:");
    for m in scene.materials.iter().skip(1) {
        println!("  - {}", m.name);
    }

    let mut scale: i32 = 2;
    let mut screen = make_screen(&mut rl, &thread, scale);
    let mut auto_rotate = false;
    let mut last_view = (cam.yaw, cam.pitch, cam.distance);
    let mut recording: Option<u32> = None;
    let mut video_yaw = 0.0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        let status;

        if let Some(i) = recording {
            // ---------- modo video ----------
            // la camara da una vuelta completa, se acerca y se aleja,
            // y pasa por dia, atardecer y noche
            let t = i as f32 / VIDEO_FRAMES as f32;
            cam.yaw = video_yaw + t * std::f32::consts::TAU;
            cam.distance = 50.0 - 20.0 * (t * std::f32::consts::TAU).sin().abs();
            cam.clamp();
            scene.set_mode((i * 3 / VIDEO_FRAMES) as u8);
            scene.time = i as f32 / 30.0;

            screen.renderer.reset();
            for _ in 0..VIDEO_SAMPLES {
                screen.renderer.render(&mut screen.frame, &scene, &cam);
            }
            save_ppm(i, screen.renderer.width, screen.renderer.height, &screen.frame);

            if i + 1 < VIDEO_FRAMES {
                recording = Some(i + 1);
            } else {
                recording = None;
                println!("Video listo! los frames estan en la carpeta frames/");
            }
            status = format!("Grabando video: frame {} de {}", i + 1, VIDEO_FRAMES);
        } else {
            scene.time += dt;
            let mut changed = false;

            // ---------- controles ----------
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                auto_rotate = !auto_rotate;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_N) {
                scene.set_mode((scene.mode + 1) % 3);
                changed = true;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_R) {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                scene.regenerate(seed);
                changed = true;
            }

            // rotacion del diorama
            if auto_rotate {
                cam.yaw += 0.25 * dt;
            }
            if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT) {
                cam.yaw -= 1.5 * dt;
            }
            if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) {
                cam.yaw += 1.5 * dt;
            }
            if rl.is_key_down(KeyboardKey::KEY_UP) {
                cam.pitch += 1.0 * dt;
            }
            if rl.is_key_down(KeyboardKey::KEY_DOWN) {
                cam.pitch -= 1.0 * dt;
            }
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                let delta = rl.get_mouse_delta();
                cam.yaw -= delta.x * 0.005;
                cam.pitch += delta.y * 0.005;
            }

            // zoom
            if rl.is_key_down(KeyboardKey::KEY_W) {
                cam.distance -= 25.0 * dt;
            }
            if rl.is_key_down(KeyboardKey::KEY_S) {
                cam.distance += 25.0 * dt;
            }
            cam.distance -= rl.get_mouse_wheel_move() * 3.0;
            cam.clamp();

            // si la camara se movio hay que empezar a promediar de nuevo
            let view = (cam.yaw, cam.pitch, cam.distance);
            if view != last_view {
                changed = true;
            }
            last_view = view;

            // calidad: menos pixeles = mas fps
            let mut new_scale = scale;
            if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
                new_scale = 4;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
                new_scale = 3;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
                new_scale = 2;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
                new_scale = 1;
            }

            // empezar a grabar el video en calidad maxima
            if rl.is_key_pressed(KeyboardKey::KEY_V) {
                let _ = std::fs::create_dir_all("frames");
                new_scale = 1;
                video_yaw = cam.yaw;
                recording = Some(0);
            }

            if new_scale != scale {
                scale = new_scale;
                screen = make_screen(&mut rl, &thread, scale);
            }

            if changed {
                screen.renderer.reset();
            }
            screen.renderer.render(&mut screen.frame, &scene, &cam);

            status = format!(
                "{}x{}  |  {}  |  muestras {}  |  semilla {}",
                screen.renderer.width,
                screen.renderer.height,
                MODE_NAMES[scene.mode as usize],
                screen.renderer.samples,
                seed
            );
        }

        // ---------- dibujar ----------
        let _ = screen.texture.update_texture(&screen.frame);
        let w = screen.renderer.width as f32;
        let h = screen.renderer.height as f32;

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &screen.texture,
            Rectangle::new(0.0, 0.0, w, h),
            Rectangle::new(0.0, 0.0, WIN_W as f32, WIN_H as f32),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
        d.draw_fps(10, 10);
        d.draw_text(&status, 10, 32, 16, Color::WHITE);
        if recording.is_none() {
            d.draw_text("A/D o mouse: girar   W/S o rueda: zoom   flechas: inclinar   espacio: auto-giro", 10, WIN_H - 44, 16, Color::WHITE);
            d.draw_text("N: dia/atardecer/noche   R: nuevo terreno   1-4: calidad   V: grabar video", 10, WIN_H - 24, 16, Color::WHITE);
        }
    }
}
