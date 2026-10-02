use crate::framebuffer::{Color, FrameBuffer};
use crate::vector::Vec3;
use std::ops::{Add, Mul};

pub const CAPAS: usize = 5;

#[derive(Clone, Copy)]
pub struct Capas(pub [Vec3; CAPAS]);

impl Capas {
    pub fn cero() -> Self {
        Capas([Vec3::new(0.0, 0.0, 0.0); CAPAS])
    }

    pub fn en(capa: usize, color: Vec3) -> Self {
        let mut capas = Capas::cero();
        capas.0[capa] = color;
        capas
    }

    pub fn a_colores(&self) -> [Color; CAPAS] {
        self.0.map(a_color)
    }
}

pub type Pixel = ([Color; CAPAS], bool);

impl Add for Capas {
    type Output = Capas;

    fn add(self, otras: Capas) -> Capas {
        Capas(std::array::from_fn(|i| self.0[i] + otras.0[i]))
    }
}

impl Mul<f32> for Capas {
    type Output = Capas;

    fn mul(self, k: f32) -> Capas {
        Capas(self.0.map(|c| c * k))
    }
}

pub fn mezclar(a: Capas, b: Capas, t: f32) -> Capas {
    a * (1.0 - t) + b * t
}

fn a_byte(valor: f32) -> u8 {
    (valor.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

fn a_color(c: Vec3) -> Color {
    Color {
        r: a_byte(c.x),
        g: a_byte(c.y),
        b: a_byte(c.z),
    }
}

pub struct Imagen {
    ancho: usize,
    alto: usize,
    capas: Vec<[Color; CAPAS]>,
    ve_piso: Vec<bool>,
}

impl Imagen {
    pub fn nueva(ancho: usize, alto: usize) -> Self {
        let negro = Color { r: 0, g: 0, b: 0 };
        Imagen {
            ancho,
            alto,
            capas: vec![[negro; CAPAS]; ancho * alto],
            ve_piso: vec![false; ancho * alto],
        }
    }

    pub fn dimensiones(&self) -> (usize, usize) {
        (self.ancho, self.alto)
    }

    pub fn pixel(&self, i: usize) -> Pixel {
        (self.capas[i], self.ve_piso[i])
    }

    pub fn fijar(&mut self, i: usize, pixel: Pixel) {
        (self.capas[i], self.ve_piso[i]) = pixel;
    }

    pub fn ve_piso(&self) -> &[bool] {
        &self.ve_piso
    }

    pub fn componer(&self, fb: &mut FrameBuffer, encendido: &[f32; CAPAS]) {
        fb.clear();
        for (i, capas) in self.capas.iter().enumerate() {
            let canal = |de: fn(&Color) -> u8| {
                let suma: f32 = capas.iter().zip(encendido).map(|(c, k)| de(c) as f32 * k).sum();
                (suma + 0.5).min(255.0) as u8
            };
            let color = Color {
                r: canal(|c| c.r),
                g: canal(|c| c.g),
                b: canal(|c| c.b),
            };
            fb.point(i % self.ancho, i / self.ancho, color);
        }
    }
}
