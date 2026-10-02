use crate::diorama::Diorama;
use crate::formas::{barra, caja_redondeada, capsula, cilindro, cono_redondeado, toro};
use crate::material::{AMARILLO, CROMO, NEGRO};
use crate::vector::Vec3;

const CUBOS_POR_UNIDAD: f32 = 4.0;
const CENTRO: [f32; 3] = [15.5, 14.0, 8.5];
const ESCALA_MUNDO: f32 = 0.12;

pub fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

pub fn a_mundo(p: Vec3) -> Vec3 {
    v(
        (p.x - CENTRO[0]) * ESCALA_MUNDO,
        (p.y - CENTRO[1]) * ESCALA_MUNDO,
        (p.z - CENTRO[2]) * ESCALA_MUNDO,
    )
}

#[derive(Clone, Copy)]
pub struct Marco {
    origen: Vec3,
    sen: f32,
    cos: f32,
}

impl Marco {
    pub fn nuevo(origen: Vec3, angulo: f32) -> Self {
        let (sen, cos) = angulo.sin_cos();
        Marco { origen, sen, cos }
    }

    pub fn local(&self, p: Vec3) -> Vec3 {
        let q = p - self.origen;
        v(q.x * self.cos - q.z * self.sen, q.y, q.x * self.sen + q.z * self.cos)
    }

    pub fn escena(&self, q: Vec3) -> Vec3 {
        self.origen + v(q.x * self.cos + q.z * self.sen, q.y, q.z * self.cos - q.x * self.sen)
    }

    pub fn envolvente(&self, min: Vec3, max: Vec3) -> (Vec3, Vec3) {
        let mut desde = v(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut hasta = v(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for esquina in 0..8 {
            let q = v(
                if esquina & 1 == 0 { min.x } else { max.x },
                if esquina & 2 == 0 { min.y } else { max.y },
                if esquina & 4 == 0 { min.z } else { max.z },
            );
            let p = self.escena(q);
            desde = v(desde.x.min(p.x), desde.y.min(p.y), desde.z.min(p.z));
            hasta = v(hasta.x.max(p.x), hasta.y.max(p.y), hasta.z.max(p.z));
        }
        (desde, hasta)
    }
}

pub fn largo_a_mundo(largo: f32) -> f32 {
    largo * ESCALA_MUNDO
}

#[derive(Clone, Copy)]
pub enum Op {
    Sumar(usize),
    Restar,
    Pintar(usize),
}

pub struct Modelo {
    diorama: Diorama,
    minimo: Vec3,
}

impl Modelo {
    pub fn nuevo(min: Vec3, max: Vec3) -> Self {
        let tamano = max - min;
        let dim = [
            (tamano.x * CUBOS_POR_UNIDAD).ceil() as i32,
            (tamano.y * CUBOS_POR_UNIDAD).ceil() as i32,
            (tamano.z * CUBOS_POR_UNIDAD).ceil() as i32,
        ];
        let tam = ESCALA_MUNDO / CUBOS_POR_UNIDAD;
        Modelo {
            diorama: Diorama::nuevo(dim, a_mundo(min), tam, 1.0),
            minimo: min,
        }
    }

    pub fn terminar(mut self) -> Diorama {
        self.diorama.preparar();
        self.diorama
    }

    fn paso() -> f32 {
        1.0 / CUBOS_POR_UNIDAD
    }

    fn centro_celda(&self, c: [i32; 3]) -> Vec3 {
        let p = Self::paso();
        self.minimo + v(c[0] as f32 + 0.5, c[1] as f32 + 0.5, c[2] as f32 + 0.5) * p
    }

    pub fn aplicar(&mut self, op: Op, min: Vec3, max: Vec3, forma: impl Fn(Vec3) -> f32) {
        let p = Self::paso();
        let margen = 3.0 * p;
        let dim = self.diorama.dim();
        let minimo = self.minimo;
        let desde = |valor: f32, eje: usize| ((((valor - margen - minimo.eje(eje)) / p).floor()) as i32).max(0);
        let hasta = |valor: f32, eje: usize| ((((valor + margen - minimo.eje(eje)) / p).ceil()) as i32).min(dim[eje]);
        for z in desde(min.z, 2)..hasta(max.z, 2) {
            for y in desde(min.y, 1)..hasta(max.y, 1) {
                for x in desde(min.x, 0)..hasta(max.x, 0) {
                    let c = [x, y, z];
                    let d = forma(self.centro_celda(c));
                    let distancia = self.diorama.distancia(c);
                    let material = self.diorama.obtener(c);
                    match op {
                        Op::Sumar(m) => {
                            let nuevo = if d <= 0.0 { Some(m) } else { material };
                            self.diorama.fijar(c, distancia.min(d), nuevo);
                        }
                        Op::Restar => {
                            let nueva = distancia.max(-d);
                            self.diorama.fijar(c, nueva, if nueva > 0.0 { None } else { material });
                        }
                        Op::Pintar(m) => {
                            if d <= 0.0 && material.is_some() {
                                self.diorama.fijar(c, distancia, Some(m));
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn caja(&mut self, op: Op, min: Vec3, max: Vec3, radio: f32) {
        self.aplicar(op, min, max, |p| caja_redondeada(p, min, max, radio));
    }

    pub fn cilindro(&mut self, op: Op, centro: Vec3, eje: usize, radio: f32, largo: f32, redondeo: f32) {
        let medio = Vec3::sobre_eje(eje, largo * 0.5 - radio) + v(radio, radio, radio);
        self.aplicar(op, centro - medio, centro + medio, |p| cilindro(p, centro, eje, radio, largo, redondeo));
    }

    pub fn capsula(&mut self, op: Op, a: Vec3, b: Vec3, radio: f32) {
        let min = v(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)) - v(radio, radio, radio);
        let max = v(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)) + v(radio, radio, radio);
        self.aplicar(op, min, max, |p| capsula(p, a, b, radio));
    }

    pub fn toro(&mut self, op: Op, centro: Vec3, eje: usize, radio_mayor: f32, radio_menor: f32) {
        let r = radio_mayor + radio_menor;
        let min = centro - v(r, r, r);
        let max = centro + v(r, r, r);
        self.aplicar(op, min, max, |p| toro(p, centro, eje, radio_mayor, radio_menor));
    }

    pub fn barra(&mut self, op: Op, a: Vec3, b: Vec3, radio: f32, redondeo: f32) {
        let min = v(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)) - v(radio, radio, radio);
        let max = v(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)) + v(radio, radio, radio);
        self.aplicar(op, min, max, |p| barra(p, a, b, radio, redondeo));
    }

    pub fn caja_en(&mut self, op: Op, marco: Marco, min: Vec3, max: Vec3, radio: f32) {
        let (desde, hasta) = marco.envolvente(min, max);
        self.aplicar(op, desde, hasta, |p| caja_redondeada(marco.local(p), min, max, radio));
    }

    pub fn gota(&mut self, op: Op, a: Vec3, b: Vec3, ra: f32, rb: f32) {
        let min = v((a.x - ra).min(b.x - rb), (a.y - ra).min(b.y - rb), (a.z - ra).min(b.z - rb));
        let max = v((a.x + ra).max(b.x + rb), (a.y + ra).max(b.y + rb), (a.z + ra).max(b.z + rb));
        self.aplicar(op, min, max, |p| cono_redondeado(p, a, b, ra, rb));
    }

    pub fn cofre(&mut self, base: Vec3, angulo: f32, escala: f32) {
        let marco = Marco::nuevo(base, angulo);
        let local = move |p: Vec3| marco.local(p) * (1.0 / escala);
        let alcance = 2.8 * escala;
        let min = base - v(alcance, 0.1, alcance);
        let max = base + v(alcance, 3.6 * escala, alcance);

        self.aplicar(Op::Sumar(AMARILLO), min, max, |p| {
            let q = local(p);
            let cuerpo = caja_redondeada(q, v(-2.0, 0.0, -1.3), v(2.0, 2.0, 1.3), 0.15);
            let tapa = cilindro(q, v(0.0, 2.0, 0.0), 0, 1.3, 4.0, 0.15).max(2.0 - q.y);
            cuerpo.min(tapa) * escala
        });
        self.aplicar(Op::Sumar(NEGRO), min, max, |p| {
            let q = local(p);
            let mut d = caja_redondeada(q, v(-2.08, 1.85, -1.38), v(2.08, 2.12, 1.38), 0.05);
            for x0 in [-1.35, 1.35] {
                let fleje = caja_redondeada(q, v(x0 - 0.2, -0.02, -1.4), v(x0 + 0.2, 2.05, 1.4), 0.05);
                let arco = cilindro(q, v(x0, 2.0, 0.0), 0, 1.4, 0.4, 0.05).max(2.0 - q.y);
                d = d.min(fleje).min(arco);
            }
            d * escala
        });
        self.aplicar(Op::Sumar(CROMO), min, max, |p| {
            caja_redondeada(local(p), v(-0.4, 1.3, 1.28), v(0.4, 2.35, 1.55), 0.1) * escala
        });
    }
}
