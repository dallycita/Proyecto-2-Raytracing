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
use scene::Scene;

const WIN_W: i32 = 960;
const WIN_H: i32 = 540;

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

    // la imagen se calcula mas pequeña y luego se estira a la ventana
    let mut scale: i32 = 3;
    let mut width = (WIN_W / scale) as usize;
    let mut height = (WIN_H / scale) as usize;
    let mut frame = vec![0u8; width * height * 4];
    let mut screen = make_texture(&mut rl, &thread, width, height);
    let mut auto_rotate = true;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        scene.time += dt;

        // ---------- controles ----------
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            auto_rotate = !auto_rotate;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_N) {
            scene.night = !scene.night;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            scene.regenerate(seed);
        }

        // rotacion del diorama
        if auto_rotate {
            cam.yaw += 0.3 * dt;
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
            cam.distance -= 20.0 * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            cam.distance += 20.0 * dt;
        }
        cam.distance -= rl.get_mouse_wheel_move() * 2.5;
        cam.clamp();

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
        if new_scale != scale {
            scale = new_scale;
            width = (WIN_W / scale) as usize;
            height = (WIN_H / scale) as usize;
            frame = vec![0u8; width * height * 4];
            screen = make_texture(&mut rl, &thread, width, height);
        }

        // ---------- render ----------
        render::render(&mut frame, width, height, &scene, &cam);
        let _ = screen.update_texture(&frame);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &screen,
            Rectangle::new(0.0, 0.0, width as f32, height as f32),
            Rectangle::new(0.0, 0.0, WIN_W as f32, WIN_H as f32),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
        d.draw_fps(10, 10);
        d.draw_text(&format!("{}x{}  |  semilla {}", width, height, seed), 10, 32, 16, Color::WHITE);
        d.draw_text("A/D girar   W/S o rueda zoom   flechas inclinar   espacio auto-giro", 10, WIN_H - 44, 16, Color::WHITE);
        d.draw_text("N dia/noche   R nuevo terreno   1/2/3 calidad", 10, WIN_H - 24, 16, Color::WHITE);
    }
}

// textura del tamaño del render donde se sube cada frame
fn make_texture(rl: &mut RaylibHandle, thread: &RaylibThread, w: usize, h: usize) -> Texture2D {
    let image = Image::gen_image_color(w as i32, h as i32, Color::BLACK);
    rl.load_texture_from_image(thread, &image)
        .expect("no se pudo crear la textura")
}
