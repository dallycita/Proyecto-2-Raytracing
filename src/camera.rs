use crate::vec3::Vec3;

// camara que gira alrededor del diorama (orbita) y se puede acercar/alejar
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub fov: f32,
}

impl OrbitCamera {
    pub fn new(target: Vec3) -> OrbitCamera {
        OrbitCamera {
            target,
            yaw: 0.8,
            pitch: 0.55,
            distance: 38.0,
            fov: 50f32.to_radians(),
        }
    }

    // posicion de la camara usando coordenadas esfericas
    pub fn eye(&self) -> Vec3 {
        let offset = Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        );
        self.target + offset * self.distance
    }

    // vectores adelante, derecha y arriba de la camara
    pub fn basis(&self) -> (Vec3, Vec3, Vec3) {
        let forward = (self.target - self.eye()).normalize();
        let right = forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(forward);
        (forward, right, up)
    }

    // limites para no meterse al terreno ni verlo desde abajo
    pub fn clamp(&mut self) {
        self.pitch = self.pitch.clamp(0.05, 1.45);
        self.distance = self.distance.clamp(16.0, 70.0);
    }
}
