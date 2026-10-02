use crate::vector::Vec3;

#[derive(Clone, Copy)]
pub struct Intersect {
    pub distancia: f32,
    pub punto: Vec3,
    pub normal: Vec3,
    pub normal_suave: Vec3,
    pub uv: (f32, f32),
    pub material: usize,
    pub pieza: usize,
    pub is_intersecting: bool,
}

impl Intersect {
    pub fn nuevo(distancia: f32, punto: Vec3, normal: Vec3, normal_suave: Vec3, uv: (f32, f32), material: usize) -> Self {
        Intersect {
            distancia,
            punto,
            normal,
            normal_suave,
            uv,
            material,
            pieza: 0,
            is_intersecting: true,
        }
    }

    pub fn vacio() -> Self {
        let cero = Vec3::new(0.0, 0.0, 0.0);
        Intersect {
            distancia: f32::INFINITY,
            punto: cero,
            normal: cero,
            normal_suave: cero,
            uv: (0.0, 0.0),
            material: 0,
            pieza: 0,
            is_intersecting: false,
        }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, origen: &Vec3, direccion: &Vec3) -> Intersect;
}
