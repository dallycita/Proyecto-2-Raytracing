use crate::material::{create_materials, Material};
use crate::skybox::{moon_dir, sun_dir, Skybox};
use crate::texture::{create_all, Texture};
use crate::vec3::Vec3;
use crate::world::{World, AIR, D, W};

// todo lo que necesita el raytracer para dibujar
pub struct Scene {
    pub world: World,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub sky_day: Skybox,
    pub sky_night: Skybox,
    pub lights: Vec<Vec3>,
    pub night: bool,
    pub time: f32,
}

impl Scene {
    pub fn new(seed: u32) -> Scene {
        let world = World::new(seed);
        let lights = world.lamps();
        Scene {
            world,
            materials: create_materials(),
            textures: create_all(),
            sky_day: Skybox::new(false),
            sky_night: Skybox::new(true),
            lights,
            night: false,
            time: 0.0,
        }
    }

    // terreno nuevo con otra semilla
    pub fn regenerate(&mut self, seed: u32) {
        self.world = World::new(seed);
        self.lights = self.world.lamps();
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(W as f32 / 2.0, 5.0, D as f32 / 2.0)
    }

    pub fn sky(&self, dir: Vec3) -> Vec3 {
        if self.night {
            self.sky_night.sample(dir)
        } else {
            self.sky_day.sample(dir)
        }
    }

    // (direccion de la luz, color de la luz, luz ambiente)
    pub fn sun(&self) -> (Vec3, Vec3, Vec3) {
        if self.night {
            (moon_dir(), Vec3::new(0.35, 0.4, 0.6), Vec3::new(0.05, 0.06, 0.1))
        } else {
            (sun_dir(), Vec3::new(1.0, 0.95, 0.85), Vec3::new(0.38, 0.4, 0.45))
        }
    }

    // en la noche las lamparas alumbran mas
    pub fn lamp_power(&self) -> f32 {
        if self.night { 2.2 } else { 0.8 }
    }

    pub fn ior_of(&self, medium: u8) -> f32 {
        if medium == AIR {
            1.0
        } else {
            self.materials[medium as usize].ior
        }
    }
}
