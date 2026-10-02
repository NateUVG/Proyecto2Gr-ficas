use crate::ray_intersect::{Intersect, RayIntersect};
use crate::vector::Vec3;

const LEJOS: f32 = 1e3;
const BLOQUE: i32 = 8;

pub struct Diorama {
    dim: [i32; 3],
    celdas: Vec<u8>,
    distancias: Vec<f32>,
    origen: Vec3,
    tam: f32,
    escala_uv: f32,
    bloques_dim: [i32; 3],
    bloques_ocupados: Vec<bool>,
    ocupado: ([i32; 3], [i32; 3]),
}

impl Diorama {
    pub fn nuevo(dim: [i32; 3], origen: Vec3, tam: f32, escala_uv: f32) -> Self {
        let n = (dim[0] * dim[1] * dim[2]) as usize;
        let bloques_dim = [
            (dim[0] + BLOQUE - 1) / BLOQUE,
            (dim[1] + BLOQUE - 1) / BLOQUE,
            (dim[2] + BLOQUE - 1) / BLOQUE,
        ];
        Diorama {
            dim,
            celdas: vec![0; n],
            distancias: vec![LEJOS; n],
            origen,
            tam,
            escala_uv,
            bloques_dim,
            bloques_ocupados: vec![true; (bloques_dim[0] * bloques_dim[1] * bloques_dim[2]) as usize],
            ocupado: ([0, 0, 0], [dim[0] - 1, dim[1] - 1, dim[2] - 1]),
        }
    }

    pub fn preparar(&mut self) {
        self.bloques_ocupados.fill(false);
        let (mut desde, mut hasta) = (self.dim, [-1; 3]);
        for z in 0..self.dim[2] {
            for y in 0..self.dim[1] {
                for x in 0..self.dim[0] {
                    let i = ((z * self.dim[1] + y) * self.dim[0] + x) as usize;
                    if self.celdas[i] != 0 {
                        let b = self.indice_bloque([x, y, z]);
                        self.bloques_ocupados[b] = true;
                        for (a, c) in [x, y, z].into_iter().enumerate() {
                            desde[a] = desde[a].min(c);
                            hasta[a] = hasta[a].max(c);
                        }
                    }
                }
            }
        }
        self.ocupado = (desde, hasta);
    }

    fn indice_bloque(&self, c: [i32; 3]) -> usize {
        let b = [c[0] / BLOQUE, c[1] / BLOQUE, c[2] / BLOQUE];
        ((b[2] * self.bloques_dim[1] + b[1]) * self.bloques_dim[0] + b[0]) as usize
    }

    fn indice(&self, c: [i32; 3]) -> Option<usize> {
        if (0..3).all(|a| c[a] >= 0 && c[a] < self.dim[a]) {
            Some(((c[2] * self.dim[1] + c[1]) * self.dim[0] + c[0]) as usize)
        } else {
            None
        }
    }

    pub fn dim(&self) -> [i32; 3] {
        self.dim
    }

    pub fn obtener(&self, c: [i32; 3]) -> Option<usize> {
        self.indice(c)
            .and_then(|i| self.celdas[i].checked_sub(1))
            .map(|m| m as usize)
    }

    pub fn distancia(&self, c: [i32; 3]) -> f32 {
        self.indice(c).map_or(LEJOS, |i| self.distancias[i])
    }

    pub fn fijar(&mut self, c: [i32; 3], distancia: f32, material: Option<usize>) {
        if let Some(i) = self.indice(c) {
            self.distancias[i] = distancia;
            self.celdas[i] = material.map_or(0, |m| m as u8 + 1);
        }
    }

    fn normal_suave(&self, c: [i32; 3], cara: Vec3) -> Vec3 {
        let diferencia = |eje: usize| {
            let (mut antes, mut despues) = (c, c);
            antes[eje] -= 1;
            despues[eje] += 1;
            self.distancia(despues) - self.distancia(antes)
        };
        let gradiente = Vec3::new(diferencia(0), diferencia(1), diferencia(2));
        let largo = gradiente.length();
        if largo < 1e-6 {
            return cara;
        }
        let normal = gradiente * (1.0 / largo);
        if normal.dot(cara) > 0.05 { normal } else { cara }
    }

    fn franjas(&self, o: Vec3, d: Vec3) -> Option<(f32, f32, usize)> {
        let mut t_entrada = f32::NEG_INFINITY;
        let mut t_salida = f32::INFINITY;
        let mut eje_entrada = 0;
        for a in 0..3 {
            let (oa, da) = (o.eje(a), d.eje(a));
            let lo = self.origen.eje(a);
            let hi = lo + self.dim[a] as f32 * self.tam;
            if da.abs() < 1e-12 {
                if oa < lo || oa > hi {
                    return None;
                }
                continue;
            }
            let (mut t0, mut t1) = ((lo - oa) / da, (hi - oa) / da);
            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
            }
            if t0 > t_entrada {
                t_entrada = t0;
                eje_entrada = a;
            }
            t_salida = t_salida.min(t1);
        }
        if t_entrada > t_salida || t_salida < 0.0 {
            return None;
        }
        Some((t_entrada, t_salida, eje_entrada))
    }

    pub fn caja(&self) -> (Vec3, Vec3) {
        let (desde, hasta) = self.ocupado;
        let borde = |c: [i32; 3]| Vec3::new(c[0] as f32, c[1] as f32, c[2] as f32) * self.tam + self.origen;
        (borde(desde), borde([hasta[0] + 1, hasta[1] + 1, hasta[2] + 1]))
    }

    pub fn entrada(&self, origen: Vec3, direccion: Vec3) -> Option<f32> {
        self.franjas(origen, direccion).map(|(t, _, _)| t.max(0.0))
    }

    fn impacto(&self, origen: Vec3, direccion: Vec3, t: f32, cara: Vec3, celda: [i32; 3], material: usize) -> Intersect {
        let punto = origen + direccion * t;
        let (u, v) = if cara.x != 0.0 {
            (punto.z, punto.y)
        } else if cara.y != 0.0 {
            (punto.x, punto.z)
        } else {
            (punto.x, punto.y)
        };
        Intersect::nuevo(
            t,
            punto,
            cara,
            self.normal_suave(celda, cara),
            (u / self.escala_uv, -v / self.escala_uv),
            material,
        )
    }
}

impl RayIntersect for Diorama {
    fn ray_intersect(&self, origen: &Vec3, direccion: &Vec3) -> Intersect {
        let (o, d) = (*origen, *direccion);
        let Some((t_entrada, t_salida, eje_entrada)) = self.franjas(o, d) else {
            return Intersect::vacio();
        };

        let t_inicio = t_entrada.max(0.0);
        let p = o + d * t_inicio;
        let mut celda = [0i32; 3];
        let mut paso = [0i32; 3];
        let mut t_max = [f32::INFINITY; 3];
        let mut t_delta = [f32::INFINITY; 3];
        let proximo_borde = |a: usize, c: i32, paso: i32| {
            let lo = self.origen.eje(a);
            match paso {
                1 => (lo + (c + 1) as f32 * self.tam - o.eje(a)) / d.eje(a),
                -1 => (lo + c as f32 * self.tam - o.eje(a)) / d.eje(a),
                _ => f32::INFINITY,
            }
        };
        for a in 0..3 {
            let lo = self.origen.eje(a);
            celda[a] = (((p.eje(a) - lo) / self.tam).floor() as i32).clamp(0, self.dim[a] - 1);
            let da = d.eje(a);
            if da > 0.0 {
                paso[a] = 1;
                t_delta[a] = self.tam / da;
            } else if da < 0.0 {
                paso[a] = -1;
                t_delta[a] = -self.tam / da;
            }
            t_max[a] = proximo_borde(a, celda[a], paso[a]);
        }

        let primera = self.obtener(celda);
        let medio = if t_entrada > 0.0 {
            if let Some(m) = primera {
                let cara = Vec3::sobre_eje(eje_entrada, -(paso[eje_entrada] as f32));
                return self.impacto(o, d, t_inicio, cara, celda, m);
            }
            None
        } else {
            primera
        };

        loop {
            if medio.is_none() && !self.bloques_ocupados[self.indice_bloque(celda)] {
                let mut t_salida_bloque = f32::INFINITY;
                let mut eje = 0;
                for a in 0..3 {
                    if paso[a] == 0 {
                        continue;
                    }
                    let base = celda[a] / BLOQUE * BLOQUE;
                    let borde = if paso[a] > 0 { base + BLOQUE - 1 } else { base };
                    let t = proximo_borde(a, borde, paso[a]);
                    if t < t_salida_bloque {
                        t_salida_bloque = t;
                        eje = a;
                    }
                }
                if t_salida_bloque > t_salida {
                    return Intersect::vacio();
                }
                let punto = o + d * t_salida_bloque;
                for a in 0..3 {
                    let base = celda[a] / BLOQUE * BLOQUE;
                    celda[a] = if a == eje {
                        if paso[a] > 0 { base + BLOQUE } else { base - 1 }
                    } else {
                        let c = ((punto.eje(a) - self.origen.eje(a)) / self.tam).floor() as i32;
                        c.clamp(base, (base + BLOQUE - 1).min(self.dim[a] - 1))
                    };
                }
                if celda[eje] < 0 || celda[eje] >= self.dim[eje] {
                    return Intersect::vacio();
                }
                for a in 0..3 {
                    t_max[a] = proximo_borde(a, celda[a], paso[a]);
                }
                if let Some(m) = self.obtener(celda) {
                    return self.impacto(o, d, t_salida_bloque, Vec3::sobre_eje(eje, -(paso[eje] as f32)), celda, m);
                }
                continue;
            }

            let a = if t_max[0] < t_max[1] {
                if t_max[0] < t_max[2] { 0 } else { 2 }
            } else if t_max[1] < t_max[2] {
                1
            } else {
                2
            };
            let t = t_max[a];
            if t > t_salida {
                return Intersect::vacio();
            }
            let anterior = celda;
            celda[a] += paso[a];
            if celda[a] < 0 || celda[a] >= self.dim[a] {
                return Intersect::vacio();
            }
            t_max[a] += t_delta[a];

            let actual = self.obtener(celda);
            if actual != medio {
                return match (actual, medio) {
                    (Some(m), _) => self.impacto(o, d, t, Vec3::sobre_eje(a, -(paso[a] as f32)), celda, m),
                    (None, Some(m)) => self.impacto(o, d, t, Vec3::sobre_eje(a, paso[a] as f32), anterior, m),
                    (None, None) => unreachable!(),
                };
            }
        }
    }
}
