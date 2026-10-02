use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    pub fn dot(self, otro: Vec3) -> f32 {
        self.x * otro.x + self.y * otro.y + self.z * otro.z
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalizado(self) -> Vec3 {
        let l = self.length();
        Vec3::new(self.x / l, self.y / l, self.z / l)
    }

    pub fn eje(self, eje: usize) -> f32 {
        match eje {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }

    pub fn sobre_eje(eje: usize, valor: f32) -> Vec3 {
        match eje {
            0 => Vec3::new(valor, 0.0, 0.0),
            1 => Vec3::new(0.0, valor, 0.0),
            _ => Vec3::new(0.0, 0.0, valor),
        }
    }

    pub fn cross(self, otro: Vec3) -> Vec3 {
        Vec3::new(
            self.y * otro.z - self.z * otro.y,
            self.z * otro.x - self.x * otro.z,
            self.x * otro.y - self.y * otro.x,
        )
    }

    pub fn reflejar(self, normal: Vec3) -> Vec3 {
        self - normal * (2.0 * self.dot(normal))
    }

    pub fn refractar(self, normal: Vec3, eta: f32) -> Option<Vec3> {
        let cos_i = (-self.dot(normal)).clamp(0.0, 1.0);
        let sin2_t = eta * eta * (1.0 - cos_i * cos_i);
        if sin2_t > 1.0 {
            None
        } else {
            let cos_t = (1.0 - sin2_t).sqrt();
            Some(self * eta + normal * (eta * cos_i - cos_t))
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, otro: Vec3) -> Vec3 {
        Vec3::new(self.x + otro.x, self.y + otro.y, self.z + otro.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, otro: Vec3) -> Vec3 {
        Vec3::new(self.x - otro.x, self.y - otro.y, self.z - otro.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, k: f32) -> Vec3 {
        Vec3::new(self.x * k, self.y * k, self.z * k)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}
