use crate::texture::*;

// cada material tiene su propia textura y sus propios parametros
pub struct Material {
    pub name: &'static str,
    pub tex_top: usize,
    pub tex_side: usize,
    pub normal_map: Option<usize>,
    pub albedo: [f32; 2], // [peso difuso, peso especular]
    pub specular: f32,    // que tan concentrado es el brillo
    pub reflectivity: f32,
    pub transparency: f32,
    pub ior: f32,      // indice de refraccion
    pub emission: f32, // luz propia del bloque
}

// el orden tiene que ser igual a los ids de world.rs
pub fn create_materials() -> Vec<Material> {
    vec![
        // 0: aire, no se dibuja nunca, solo ocupa el lugar
        Material {
            name: "aire",
            tex_top: T_DIRT,
            tex_side: T_DIRT,
            normal_map: None,
            albedo: [0.0, 0.0],
            specular: 1.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emission: 0.0,
        },
        // 1
        Material {
            name: "pasto",
            tex_top: T_GRASS_TOP,
            tex_side: T_GRASS_SIDE,
            normal_map: None,
            albedo: [0.9, 0.1],
            specular: 10.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emission: 0.0,
        },
        // 2
        Material {
            name: "tierra",
            tex_top: T_DIRT,
            tex_side: T_DIRT,
            normal_map: None,
            albedo: [0.9, 0.05],
            specular: 5.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emission: 0.0,
        },
        // 3: la piedra usa mapa normal para que se vea con relieve
        Material {
            name: "piedra",
            tex_top: T_STONE,
            tex_side: T_STONE,
            normal_map: Some(T_STONE_NORMAL),
            albedo: [0.85, 0.25],
            specular: 25.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emission: 0.0,
        },
        // 4
        Material {
            name: "arena",
            tex_top: T_SAND,
            tex_side: T_SAND,
            normal_map: None,
            albedo: [0.9, 0.1],
            specular: 8.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emission: 0.0,
        },
        // 5: agua, transparente con refraccion y un poco de reflejo
        Material {
            name: "agua",
            tex_top: T_WATER,
            tex_side: T_WATER,
            normal_map: Some(T_WATER_NORMAL),
            albedo: [0.4, 0.6],
            specular: 80.0,
            reflectivity: 0.25,
            transparency: 0.55,
            ior: 1.33,
            emission: 0.0,
        },
        // 6
        Material {
            name: "madera",
            tex_top: T_LOG_TOP,
            tex_side: T_LOG_SIDE,
            normal_map: None,
            albedo: [0.9, 0.1],
            specular: 12.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emission: 0.0,
        },
        // 7
        Material {
            name: "hojas",
            tex_top: T_LEAVES,
            tex_side: T_LEAVES,
            normal_map: None,
            albedo: [0.9, 0.05],
            specular: 5.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emission: 0.0,
        },
        // 8: hielo en las cimas, refleja bastante
        Material {
            name: "hielo",
            tex_top: T_ICE,
            tex_side: T_ICE,
            normal_map: None,
            albedo: [0.6, 0.5],
            specular: 60.0,
            reflectivity: 0.4,
            transparency: 0.15,
            ior: 1.31,
            emission: 0.0,
        },
        // 9: lampara, brilla sola y alumbra lo que tiene cerca
        Material {
            name: "glowstone",
            tex_top: T_GLOWSTONE,
            tex_side: T_GLOWSTONE,
            normal_map: None,
            albedo: [1.0, 0.0],
            specular: 1.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emission: 1.0,
        },
    ]
}
