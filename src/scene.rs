use crate::material::{create_materials, Material};
use crate::skybox::{light_dir, Skybox, DAY, NIGHT, SUNSET};
use crate::texture::{create_all, Texture};
use crate::vec3::Vec3;
use crate::world::{Light, World, AIR, D, W};

// como esta la luz en cada momento del dia
pub struct Lighting {
    pub dir: Vec3,
    pub color: Vec3,
    pub sky_ambient: Vec3,    // luz que viene de arriba
    pub ground_ambient: Vec3, // luz que rebota del suelo
    pub lamp_scale: f32,      // en la noche las lamparas se notan mas
    pub exposure: f32,
}

// todo lo que necesita el raytracer para dibujar
pub struct Scene {
    pub world: World,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub lights: Vec<Light>,
    pub mode: u8,
    pub light: Lighting,
    pub time: f32,
    skies: Vec<Skybox>,
}

impl Scene {
    pub fn new(seed: u32) -> Scene {
        let world = World::new(seed);
        let lights = world.lights();
        Scene {
            world,
            materials: create_materials(),
            textures: create_all(),
            lights,
            mode: DAY,
            light: lighting_for(DAY),
            time: 0.0,
            skies: vec![Skybox::new(DAY), Skybox::new(SUNSET), Skybox::new(NIGHT)],
        }
    }

    // terreno nuevo con otra semilla
    pub fn regenerate(&mut self, seed: u32) {
        self.world = World::new(seed);
        self.lights = self.world.lights();
    }

    pub fn set_mode(&mut self, mode: u8) {
        self.mode = mode;
        self.light = lighting_for(mode);
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(W as f32 / 2.0, 6.0, D as f32 / 2.0)
    }

    pub fn sky(&self, dir: Vec3) -> Vec3 {
        self.skies[self.mode as usize].sample(dir)
    }

    pub fn ior_of(&self, medium: u8) -> f32 {
        if medium == AIR {
            1.0
        } else {
            self.materials[medium as usize].ior
        }
    }
}

fn lighting_for(mode: u8) -> Lighting {
    match mode {
        DAY => Lighting {
            dir: light_dir(DAY),
            color: Vec3::new(1.0, 0.92, 0.8) * 2.0,
            sky_ambient: Vec3::new(0.3, 0.4, 0.6) * 0.9,
            ground_ambient: Vec3::new(0.25, 0.22, 0.18) * 0.7,
            lamp_scale: 0.6,
            exposure: 0.95,
        },
        SUNSET => Lighting {
            dir: light_dir(SUNSET),
            color: Vec3::new(1.0, 0.55, 0.28) * 2.4,
            sky_ambient: Vec3::new(0.32, 0.28, 0.42) * 0.6,
            ground_ambient: Vec3::new(0.2, 0.12, 0.1) * 0.5,
            lamp_scale: 1.3,
            exposure: 1.1,
        },
        _ => Lighting {
            dir: light_dir(NIGHT),
            color: Vec3::new(0.45, 0.55, 0.95) * 0.3,
            sky_ambient: Vec3::new(0.05, 0.07, 0.14),
            ground_ambient: Vec3::new(0.02, 0.02, 0.03),
            lamp_scale: 2.5,
            exposure: 1.5,
        },
    }
}
