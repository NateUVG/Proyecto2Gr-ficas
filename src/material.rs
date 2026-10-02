use crate::textura::Textura;

pub struct Material {
    pub textura: Textura,
    pub albedo: f32,
    pub especular: f32,
    pub transparencia: f32,
    pub reflectividad: f32,
    pub indice_refraccion: f32,
}

impl Material {
    pub fn nuevo(
        ruta_textura: &str,
        albedo: f32,
        especular: f32,
        transparencia: f32,
        reflectividad: f32,
        indice_refraccion: f32,
    ) -> Self {
        Material {
            textura: Textura::cargar(ruta_textura),
            albedo,
            especular,
            transparencia,
            reflectividad,
            indice_refraccion,
        }
    }
}

pub const AZUL: usize = 0;
pub const VIDRIO: usize = 1;
pub const NEGRO: usize = 2;
pub const AMARILLO: usize = 3;
pub const CROMO: usize = 4;

pub fn materiales() -> Vec<Material> {
    vec![
        Material::nuevo("assets/azul.png", 0.95, 0.55, 0.0, 0.15, 1.0),
        Material::nuevo("assets/vidrio.png", 0.35, 0.9, 0.35, 0.12, 1.5),
        Material::nuevo("assets/negro.png", 0.85, 0.12, 0.0, 0.0, 1.0),
        Material::nuevo("assets/amarillo.png", 0.95, 0.6, 0.0, 0.2, 1.0),
        Material::nuevo("assets/cromo.png", 0.6, 1.0, 0.0, 0.45, 1.0),
    ]
}
