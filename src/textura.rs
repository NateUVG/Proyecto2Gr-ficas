use image::RgbImage;

use crate::vector::Vec3;

pub struct Textura {
    imagen: RgbImage,
}

impl Textura {
    pub fn cargar(ruta: &str) -> Self {
        let imagen = image::open(ruta)
            .unwrap_or_else(|e| panic!("no se pudo cargar la textura {ruta}: {e:?}"))
            .to_rgb8();
        Textura { imagen }
    }

    pub fn muestrear(&self, u: f32, v: f32) -> Vec3 {
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);
        let x = (u * (self.imagen.width() - 1) as f32) as u32;
        let y = (v * (self.imagen.height() - 1) as f32) as u32;
        let px = self.imagen.get_pixel(x, y);
        Vec3::new(px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0)
    }
}
