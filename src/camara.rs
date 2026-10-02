use crate::vector::Vec3;

pub struct Camara {
    pub objetivo: Vec3,
    pub distancia: f32,
    pub azimut: f32,
    pub elevacion: f32,
    distancia_min: f32,
    distancia_max: f32,
}

impl Camara {
    pub fn nueva(objetivo: Vec3, distancia: f32, azimut: f32, elevacion: f32, distancia_min: f32, distancia_max: f32) -> Self {
        Camara {
            objetivo,
            distancia,
            azimut,
            elevacion,
            distancia_min,
            distancia_max,
        }
    }

    pub fn rotar(&mut self, delta_azimut: f32, delta_elevacion: f32) {
        self.azimut += delta_azimut;
        let limite = std::f32::consts::FRAC_PI_2 - 0.01;
        self.elevacion = (self.elevacion + delta_elevacion).clamp(-limite, limite);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distancia = (self.distancia + delta).clamp(self.distancia_min, self.distancia_max);
    }

    fn ojo(&self) -> Vec3 {
        Vec3::new(
            self.objetivo.x + self.distancia * self.elevacion.cos() * self.azimut.sin(),
            self.objetivo.y + self.distancia * self.elevacion.sin(),
            self.objetivo.z + self.distancia * self.elevacion.cos() * self.azimut.cos(),
        )
    }

    pub fn base_de_rayos(&self) -> (Vec3, Vec3, Vec3, Vec3) {
        let ojo = self.ojo();
        let adelante = (self.objetivo - ojo).normalizado();
        let mundo_arriba = Vec3::new(0.0, 1.0, 0.0);
        let derecha = adelante.cross(mundo_arriba).normalizado();
        let arriba = derecha.cross(adelante);
        (ojo, adelante, derecha, arriba)
    }
}
