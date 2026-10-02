use crate::diorama::Diorama;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::vector::Vec3;

#[derive(Clone, Copy)]
pub struct Vaiven {
    pub pivote: Vec3,
    pub altura: f32,
    pub inclinacion: f32,
    pub giro: f32,
    pub periodo: f32,
    pub fase: f32,
}

#[derive(Clone, Copy)]
pub struct Parpadeo {
    pub periodo: f32,
    pub apagado: f32,
    pub fase: f32,
}

const ARRANQUE: f32 = 0.6;
const APAGADO_GRADUAL: f32 = 0.25;

impl Parpadeo {
    pub fn encendido(&self, t: f32) -> f32 {
        let c = (t + self.fase).rem_euclid(self.periodo);
        let se_apaga = self.periodo - self.apagado - APAGADO_GRADUAL;
        if c < ARRANQUE {
            match c {
                c if c < 0.06 => 1.0,
                c if c < 0.16 => 0.0,
                c if c < 0.22 => 0.7,
                c if c < 0.36 => 0.05,
                c => 0.6 + 0.4 * (c - 0.36) / (ARRANQUE - 0.36),
            }
        } else if c < se_apaga {
            1.0
        } else {
            let t = ((c - se_apaga) / APAGADO_GRADUAL).min(1.0);
            1.0 - t * t * (3.0 - 2.0 * t)
        }
    }
}

#[derive(Clone, Copy)]
struct Pose {
    ejes: [Vec3; 3],
    pivote: Vec3,
    desplazamiento: Vec3,
}

impl Pose {
    fn dir_a_local(&self, d: Vec3) -> Vec3 {
        Vec3::new(self.ejes[0].dot(d), self.ejes[1].dot(d), self.ejes[2].dot(d))
    }

    fn dir_a_escena(&self, d: Vec3) -> Vec3 {
        self.ejes[0] * d.x + self.ejes[1] * d.y + self.ejes[2] * d.z
    }

    fn punto_a_local(&self, p: Vec3) -> Vec3 {
        self.dir_a_local(p - self.pivote - self.desplazamiento) + self.pivote
    }

    fn punto_a_escena(&self, p: Vec3) -> Vec3 {
        self.dir_a_escena(p - self.pivote) + self.pivote + self.desplazamiento
    }
}

fn rotacion(vector: Vec3) -> [Vec3; 3] {
    let angulo = vector.length();
    let base = [Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)];
    if angulo < 1e-6 {
        return base;
    }
    let k = vector * (1.0 / angulo);
    let (sen, cos) = angulo.sin_cos();
    base.map(|e| e * cos + k.cross(e) * sen + k * (k.dot(e) * (1.0 - cos)))
}

pub struct Pieza {
    diorama: Diorama,
    vaiven: Option<Vaiven>,
    pose: Option<Pose>,
    lente: Option<usize>,
}

impl Pieza {
    pub fn fija(diorama: Diorama) -> Self {
        Pieza { diorama, vaiven: None, pose: None, lente: None }
    }

    pub fn flotando(diorama: Diorama, vaiven: Vaiven) -> Self {
        Pieza { diorama, vaiven: Some(vaiven), pose: None, lente: None }
    }

    pub fn reflector(diorama: Diorama, capa: usize) -> Self {
        Pieza { diorama, vaiven: None, pose: None, lente: Some(capa) }
    }

    pub fn lente(&self) -> Option<usize> {
        self.lente
    }

    pub fn animar(&mut self, t: f32) {
        let Some(m) = self.vaiven else {
            return;
        };
        let w = std::f32::consts::TAU * t / m.periodo + m.fase;
        let giro = Vec3::new(
            m.inclinacion * w.sin(),
            m.giro * (0.6 * w + 0.4).sin(),
            m.inclinacion * 0.7 * (0.8 * w + 1.7).sin(),
        );
        self.pose = Some(Pose {
            ejes: rotacion(giro),
            pivote: m.pivote,
            desplazamiento: Vec3::new(0.0, m.altura * (1.3 * w).sin(), 0.0),
        });
    }

    pub fn esquinas(&self) -> Option<[Vec3; 8]> {
        self.vaiven?;
        let (min, max) = self.diorama.caja();
        let esquinas: [Vec3; 8] = std::array::from_fn(|i| {
            Vec3::new(
                if i & 1 == 0 { min.x } else { max.x },
                if i & 2 == 0 { min.y } else { max.y },
                if i & 4 == 0 { min.z } else { max.z },
            )
        });
        Some(match &self.pose {
            Some(pose) => esquinas.map(|p| pose.punto_a_escena(p)),
            None => esquinas,
        })
    }

    fn a_local(&self, origen: Vec3, direccion: Vec3) -> (Vec3, Vec3) {
        match &self.pose {
            Some(pose) => (pose.punto_a_local(origen), pose.dir_a_local(direccion)),
            None => (origen, direccion),
        }
    }

    pub fn entrada(&self, origen: Vec3, direccion: Vec3) -> Option<f32> {
        let (o, d) = self.a_local(origen, direccion);
        self.diorama.entrada(o, d)
    }

    pub fn chocar(&self, origen: Vec3, direccion: Vec3) -> Intersect {
        let (o, d) = self.a_local(origen, direccion);
        let mut hit = self.diorama.ray_intersect(&o, &d);
        if let (Some(pose), true) = (&self.pose, hit.is_intersecting) {
            hit.punto = origen + direccion * hit.distancia;
            hit.normal = pose.dir_a_escena(hit.normal);
            hit.normal_suave = pose.dir_a_escena(hit.normal_suave);
        }
        hit
    }
}
