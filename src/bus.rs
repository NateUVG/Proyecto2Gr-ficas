use crate::animacion::{Pieza, Vaiven};
use crate::diorama::Diorama;
use crate::formas::{caja_redondeada, cilindro, cono_redondeado};
use crate::material::{AMARILLO, AZUL, CROMO, NEGRO, VIDRIO};
use crate::modelo::{Modelo, Op, a_mundo, v};
use crate::vector::Vec3;

const RADIO_LLANTA: f32 = 3.9;
const ALTURA_EJE: f32 = 4.4;
const EJE_TRASERO: f32 = 5.5;
const EJE_DELANTERO: f32 = 28.5;

pub const PLATAFORMA_CENTRO: (f32, f32) = (16.3, 9.0);
pub const PLATAFORMA_RADIO: f32 = 27.0;
const PLATAFORMA_TAPA: f32 = ALTURA_EJE - RADIO_LLANTA;
pub const PLATAFORMA_BASE: f32 = PLATAFORMA_TAPA - 2.8;

pub fn altura_piso() -> f32 {
    a_mundo(v(0.0, PLATAFORMA_BASE, 0.0)).y
}

pub fn centro_escena() -> Vec3 {
    a_mundo(v(PLATAFORMA_CENTRO.0, (PLATAFORMA_BASE + 56.0) * 0.5, PLATAFORMA_CENTRO.1))
}

impl Modelo {
    fn ventana(&mut self, min: Vec3, max: Vec3, eje_pared: usize, afuera: f32, hacia_afuera: f32, redondeo: f32) {
        let grosor = 1.2;
        let con_eje = |p: Vec3, valor: f32| match eje_pared {
            0 => v(valor, p.y, p.z),
            1 => v(p.x, valor, p.z),
            _ => v(p.x, p.y, valor),
        };
        let (a, b) = (afuera - hacia_afuera * grosor, afuera + hacia_afuera * 0.5);
        self.caja(Op::Pintar(VIDRIO), con_eje(min, a.min(b)), con_eje(max, a.max(b)), redondeo);
        let (a, b) = (afuera - hacia_afuera * 0.2, afuera + hacia_afuera * 0.5);
        self.caja(Op::Restar, con_eje(min, a.min(b)), con_eje(max, a.max(b)), redondeo);
    }

    fn llanta(&mut self, cx: f32, cara: f32, afuera: f32) {
        let r = RADIO_LLANTA;
        let ancho = 4.5;
        let zc = cara - afuera * ancho * 0.5;
        let eje = v(cx, ALTURA_EJE, zc);

        let arco = r + 0.8;
        let arco_min = v(cx - arco, 5.6, zc - (ancho * 0.5 + 0.5));
        let arco_max = v(cx + arco, ALTURA_EJE + arco, zc + (ancho * 0.5 + 0.5));
        self.aplicar(Op::Restar, arco_min, arco_max, |p| {
            cilindro(p, eje, 2, arco, ancho + 1.0, 0.0).max(5.6 - p.y)
        });
        let liner_max = v(cx + arco + 0.6, ALTURA_EJE + arco + 0.6, zc + ancho * 0.5 + 1.2);
        let liner_min = v(cx - arco - 0.6, 5.6, zc - ancho * 0.5 - 1.2);
        self.aplicar(Op::Sumar(NEGRO), liner_min, liner_max, |p| {
            let anillo = cilindro(p, eje, 2, arco + 0.5, ancho + 1.0, 0.0).max(-cilindro(p, eje, 2, arco, ancho + 2.0, 0.0));
            let pared = cilindro(p, eje - v(0.0, 0.0, afuera * (ancho * 0.5 + 0.8)), 2, arco + 0.5, 0.5, 0.0);
            anillo.min(pared).max(5.6 - p.y).max((p.z - 9.0).abs() - 8.4)
        });

        self.cilindro(Op::Sumar(NEGRO), eje, 2, r, ancho, 1.0);
        self.cilindro(Op::Restar, v(cx, ALTURA_EJE, cara), 2, 0.63 * r, 1.6, 0.0);
        self.cilindro(Op::Sumar(AMARILLO), v(cx, ALTURA_EJE, cara - afuera * 0.9), 2, 0.64 * r, 0.6, 0.15);
        self.cilindro(Op::Sumar(NEGRO), v(cx, ALTURA_EJE, cara - afuera * 0.75), 2, 0.45 * r, 0.3, 0.1);
        self.cilindro(Op::Sumar(AMARILLO), v(cx, ALTURA_EJE, cara - afuera * 0.7), 2, 0.41 * r, 0.5, 0.15);
        self.cilindro(Op::Sumar(CROMO), v(cx, ALTURA_EJE, cara - afuera * 0.4), 2, 0.19 * r, 0.9, 0.3);
        for k in 0..5 {
            let angulo = k as f32 * std::f32::consts::TAU / 5.0 + 0.3;
            let (dx, dy) = (0.3 * r * angulo.cos(), 0.3 * r * angulo.sin());
            let centro = v(cx + dx, ALTURA_EJE + dy, cara - afuera * 0.55);
            self.cilindro(Op::Sumar(CROMO), centro, 2, 0.06 * r, 0.5, 0.08);
        }
    }
}

pub fn construir() -> Vec<Pieza> {
    let (px, pz) = PLATAFORMA_CENTRO;
    let vaiven_globo = Vaiven {
        pivote: a_mundo(v(8.6, 23.2, 9.0)),
        altura: 0.0,
        inclinacion: 0.035,
        giro: 0.0,
        periodo: 7.0,
        fase: 0.0,
    };
    vec![
        Pieza::fija(carroceria()),
        Pieza::flotando(globo(), vaiven_globo),
        Pieza::fija(plataforma()),
        Pieza::fija(cofre(v(px - 22.5, PLATAFORMA_TAPA, pz + 6.0), 0.35)),
        Pieza::fija(cofre(v(px + 22.5, PLATAFORMA_TAPA, pz - 6.0), 0.85)),
    ]
}

fn carroceria() -> Diorama {
    let mut b = Modelo::nuevo(v(-4.5, -0.5, -1.5), v(37.0, 27.5, 21.0));

    b.caja(Op::Sumar(AZUL), v(0.0, 5.6, 1.0), v(24.0, 21.0, 17.0), 1.0);
    b.caja(Op::Sumar(AZUL), v(0.0, 15.0, 1.0), v(24.0, 24.0, 17.0), 3.0);
    b.caja(Op::Restar, v(0.8, 6.4, 1.8), v(23.2, 21.0, 16.2), 0.8);
    b.caja(Op::Restar, v(0.8, 15.0, 1.8), v(23.2, 23.2, 16.2), 2.2);
    b.caja(Op::Pintar(NEGRO), v(-1.0, 4.0, 0.0), v(25.0, 6.5, 18.0), 0.0);

    b.caja(Op::Sumar(AZUL), v(23.4, 5.6, 2.0), v(33.2, 15.0, 16.0), 2.4);
    b.caja(Op::Pintar(NEGRO), v(23.0, 13.6, 1.0), v(34.0, 16.0, 17.0), 0.0);
    b.caja(Op::Pintar(NEGRO), v(32.2, 12.6, 1.0), v(34.0, 16.0, 17.0), 0.0);
    let fender_centro = v(EJE_DELANTERO, ALTURA_EJE, 9.0);
    b.aplicar(Op::Sumar(AZUL), v(21.0, 5.6, 0.4), v(33.4, 11.5, 17.6), |p| {
        cilindro(p, fender_centro, 2, RADIO_LLANTA + 2.0, 17.0, 1.0)
            .max(5.6 - p.y)
            .max(p.x - 33.4)
    });

    for cx in [EJE_TRASERO, EJE_DELANTERO] {
        b.llanta(cx, 18.2, 1.0);
        b.llanta(cx, -0.2, -1.0);
    }

    b.caja(Op::Pintar(NEGRO), v(0.6, 14.6, 15.9), v(15.6, 21.1, 17.5), 0.0);
    for x0 in [1.1, 6.1, 11.1] {
        b.ventana(v(x0, 15.6, 0.0), v(x0 + 3.8, 20.3, 0.0), 2, 17.0, 1.0, 0.3);
    }
    b.caja(Op::Pintar(NEGRO), v(15.9, 6.2, 15.9), v(22.9, 21.1, 17.5), 0.0);
    for x0 in [16.5, 19.9] {
        b.ventana(v(x0, 15.6, 0.0), v(x0 + 2.5, 20.4, 0.0), 2, 17.0, 1.0, 0.25);
        b.ventana(v(x0, 7.4, 0.0), v(x0 + 2.5, 14.6, 0.0), 2, 17.0, 1.0, 0.25);
    }
    b.caja(Op::Restar, v(19.3, 6.4, 16.85), v(19.5, 21.0, 17.5), 0.0);
    for y in [10.8, 12.9] {
        b.caja(Op::Sumar(NEGRO), v(0.5, y, 16.6), v(15.6, y + 0.6, 17.35), 0.2);
    }

    b.caja(Op::Pintar(NEGRO), v(0.6, 14.6, 0.5), v(22.6, 21.1, 2.1), 0.0);
    for x0 in [1.1, 6.1, 11.1, 16.1] {
        b.ventana(v(x0, 15.6, 0.0), v(x0 + 3.8, 20.3, 0.0), 2, 1.0, -1.0, 0.3);
    }
    for y in [10.8, 12.9] {
        b.caja(Op::Sumar(NEGRO), v(0.5, y, 0.65), v(23.0, y + 0.6, 1.4), 0.2);
    }

    b.caja(Op::Pintar(NEGRO), v(-0.5, 14.6, 3.5), v(1.0, 21.1, 14.5), 0.0);
    b.ventana(v(0.0, 15.6, 4.2), v(0.0, 20.3, 13.8), 0, 0.0, -1.0, 0.3);
    for y in [10.8, 12.9] {
        b.caja(Op::Sumar(NEGRO), v(-0.4, y, 1.3), v(0.4, y + 0.6, 16.7), 0.2);
    }
    b.caja(Op::Sumar(NEGRO), v(-1.2, 4.8, 1.0), v(0.8, 7.6, 17.0), 0.5);

    b.caja(Op::Pintar(NEGRO), v(22.9, 14.8, 1.9), v(24.6, 21.2, 16.1), 0.0);
    b.ventana(v(0.0, 15.6, 2.6), v(0.0, 20.6, 15.4), 0, 24.0, 1.0, 0.4);

    b.caja(Op::Sumar(NEGRO), v(21.8, 20.6, 0.7), v(25.6, 24.4, 17.3), 1.3);
    b.caja(Op::Pintar(AZUL), v(25.1, 21.2, 5.4), v(26.0, 23.7, 12.6), 0.0);
    for z in [3.0, 15.0] {
        b.cilindro(Op::Sumar(NEGRO), v(25.5, 22.4, z), 0, 1.05, 0.5, 0.15);
        b.cilindro(Op::Sumar(CROMO), v(25.75, 22.4, z), 0, 0.55, 0.4, 0.15);
    }

    for a in [3.2, 11.8] {
        b.aplicar(Op::Sumar(CROMO), v(a, 18.5, 0.2), v(a + 2.2, 24.8, 17.8), |p| {
            let techo = caja_redondeada(p, v(0.0, 15.0, 1.0), v(24.0, 24.0, 17.0), 3.0);
            (techo - 0.5)
                .max(-(techo + 0.3))
                .max((p.x - (a + 1.1)).abs() - 1.1)
                .max(19.0 - p.y)
        });
        b.caja(Op::Sumar(CROMO), v(a + 0.1, 24.2, 5.5), v(a + 2.1, 25.6, 12.5), 0.6);
        b.caja(Op::Sumar(AMARILLO), v(a + 0.5, 25.3, 7.4), v(a + 1.7, 26.3, 10.6), 0.35);
        b.cilindro(Op::Sumar(CROMO), v(a + 1.1, 25.0, 9.0), 0, 0.5, 3.0, 0.1);
    }

    b.caja(Op::Sumar(CROMO), v(32.6, 8.2, 4.6), v(33.9, 14.0, 13.4), 0.5);
    b.caja(Op::Restar, v(33.4, 8.8, 5.2), v(34.2, 13.4, 12.8), 0.3);
    b.caja(Op::Sumar(NEGRO), v(32.9, 8.8, 5.2), v(33.5, 13.4, 12.8), 0.0);
    for y in [9.6, 10.6, 11.6, 12.6] {
        b.caja(Op::Sumar(CROMO), v(33.1, y - 0.17, 5.3), v(33.7, y + 0.17, 12.7), 0.1);
    }
    b.caja(Op::Sumar(CROMO), v(32.9, 14.0, 8.0), v(33.6, 14.6, 10.0), 0.2);
    for (z0, z1) in [(1.4, 4.4), (13.6, 16.6)] {
        b.caja(Op::Sumar(NEGRO), v(32.3, 10.0, z0), v(34.0, 12.2, z1), 0.5);
        b.caja(Op::Sumar(CROMO), v(33.6, 10.6, z0 + 0.6), v(34.2, 11.6, z1 - 0.6), 0.2);
    }
    b.caja(Op::Sumar(NEGRO), v(32.6, 4.6, 0.6), v(36.0, 7.8, 17.4), 0.8);

    b.caja(Op::Sumar(CROMO), v(10.5, 3.6, 13.8), v(15.2, 6.2, 17.3), 0.4);
    b.cilindro(Op::Sumar(CROMO), v(21.8, 4.8, 15.4), 0, 0.45, 3.4, 0.1);

    b.caja(Op::Sumar(NEGRO), v(21.8, 6.4, 1.8), v(23.2, 13.8, 16.2), 0.3);
    b.toro(Op::Sumar(NEGRO), v(21.4, 14.6, 4.4), 0, 1.2, 0.22);
    b.capsula(Op::Sumar(NEGRO), v(21.4, 14.6, 4.4), v(22.9, 12.2, 4.4), 0.25);
    for x in [3.0, 7.0, 11.0, 15.0, 19.0] {
        let lados: &[(f32, f32)] = if x < 15.0 { &[(2.0, 6.0), (12.0, 16.0)] } else { &[(2.0, 6.0)] };
        for &(z0, z1) in lados {
            b.caja(Op::Sumar(NEGRO), v(x, 6.4, z0), v(x + 2.2, 9.4, z1), 0.3);
            b.caja(Op::Sumar(NEGRO), v(x, 9.0, z0), v(x + 0.8, 14.0, z1), 0.3);
        }
    }

    b.caja(Op::Sumar(NEGRO), v(-2.6, 18.8, 13.5), v(1.0, 24.0, 18.6), 1.2);
    b.cilindro(Op::Sumar(CROMO), v(-0.8, 21.6, 18.6), 2, 1.5, 0.5, 0.2);
    b.cilindro(Op::Sumar(NEGRO), v(-0.8, 21.6, 18.85), 2, 0.5, 0.4, 0.1);
    b.caja(Op::Sumar(CROMO), v(-1.9, 12.0, 16.6), v(0.3, 19.5, 18.6), 0.6);
    b.cilindro(Op::Sumar(NEGRO), v(-0.8, 11.2, 17.8), 2, 2.0, 2.8, 0.5);
    b.caja(Op::Sumar(NEGRO), v(-3.4, 9.8, 16.6), v(-0.8, 12.6, 19.0), 0.6);
    b.capsula(Op::Sumar(CROMO), v(0.3, 10.2, 18.6), v(EJE_TRASERO, ALTURA_EJE, 19.0), 0.9);
    b.cilindro(Op::Sumar(AMARILLO), v(EJE_TRASERO, ALTURA_EJE, 19.0), 2, 1.5, 1.1, 0.3);

    b.terminar()
}

fn globo() -> Diorama {
    let mut b = Modelo::nuevo(v(-3.0, 22.0, -4.0), v(23.0, 57.0, 22.0));

    let globo_x = 10.0;
    let globo_z = 9.0;
    let cima = v(globo_x, 44.0, globo_z);
    let boca = v(globo_x, 32.5, globo_z);
    let globo = move |p: Vec3| cono_redondeado(p, cima, boca, 12.0, 2.6);
    b.aplicar(Op::Sumar(AZUL), v(globo_x - 12.0, 29.5, globo_z - 12.0), v(globo_x + 12.0, 56.0, globo_z + 12.0), globo);
    for k in 0..4 {
        let angulo = k as f32 * std::f32::consts::FRAC_PI_4;
        let (sen, cos) = angulo.sin_cos();
        b.aplicar(Op::Pintar(NEGRO), v(globo_x - 12.0, 33.0, globo_z - 12.0), v(globo_x + 12.0, 56.0, globo_z + 12.0), |p| {
            let costura = ((p.x - globo_x) * sen - (p.z - globo_z) * cos).abs() - 0.22;
            costura.max(33.0 - p.y)
        });
    }
    b.cilindro(Op::Sumar(NEGRO), v(globo_x, 31.0, globo_z), 1, 2.9, 1.6, 0.3);
    b.toro(Op::Sumar(CROMO), v(globo_x, 30.3, globo_z), 1, 2.9, 0.35);
    for a in [3.2, 11.8] {
        for z in [1.6, 16.4] {
            let enganche = v(a + 1.1, 23.2, z);
            let hacia = v(enganche.x - globo_x, 0.0, enganche.z - globo_z).normalizado();
            let en_aro = v(globo_x + hacia.x * 2.9, 30.0, globo_z + hacia.z * 2.9);
            b.capsula(Op::Sumar(NEGRO), enganche, en_aro, 0.3);
        }
    }

    b.terminar()
}

fn plataforma() -> Diorama {
    let (px, pz) = PLATAFORMA_CENTRO;
    let r = PLATAFORMA_RADIO;
    let borde = r + 2.0;
    let mut b = Modelo::nuevo(
        v(px - borde, PLATAFORMA_BASE - 0.7, pz - borde),
        v(px + borde, PLATAFORMA_TAPA + 1.0, pz + borde),
    );
    b.cilindro(Op::Sumar(CROMO), v(px, PLATAFORMA_BASE + 0.55, pz), 1, r + 0.8, 1.1, 0.3);
    b.cilindro(Op::Sumar(NEGRO), v(px, PLATAFORMA_TAPA - 1.0, pz), 1, r, 2.0, 0.4);
    b.aplicar(Op::Pintar(AMARILLO), v(px - r, PLATAFORMA_TAPA - 0.5, pz - r), v(px + r, PLATAFORMA_TAPA, pz + r), |p| {
        let radial = ((p.x - px).powi(2) + (p.z - pz).powi(2)).sqrt();
        ((radial - (r - 2.5)).abs() - 0.45).max(PLATAFORMA_TAPA - 0.4 - p.y)
    });

    b.terminar()
}

fn cofre(base: Vec3, angulo: f32) -> Diorama {
    let escala = 1.35;
    let alcance = 2.8 * escala + 1.0;
    let mut b = Modelo::nuevo(base - v(alcance, 1.0, alcance), base + v(alcance, 3.6 * escala + 1.0, alcance));
    b.cofre(base, angulo, escala);
    b.terminar()
}
