use crate::animacion::{Parpadeo, Pieza, Vaiven};
use crate::bus::{PLATAFORMA_BASE, PLATAFORMA_CENTRO};
use crate::diorama::Diorama;
use crate::formas::{barra, caja_redondeada};
use crate::material::{AMARILLO, AZUL, CROMO, NEGRO, VIDRIO};
use crate::modelo::{Marco, Modelo, Op, a_mundo, largo_a_mundo, v};
use crate::vector::Vec3;

const PISO: f32 = PLATAFORMA_BASE;

fn alrededor(derecha: f32, hacia_camara: f32, y: f32) -> Vec3 {
    let (sen, cos) = crate::AZIMUT_INICIAL.sin_cos();
    let (px, pz) = PLATAFORMA_CENTRO;
    v(px + derecha * cos + hacia_camara * sen, y, pz - derecha * sen + hacia_camara * cos)
}

const INTENSIDAD_REFLECTOR: f32 = 0.6;

pub struct Reflector {
    pub posicion: Vec3,
    pub eje: Vec3,
    pub intensidad: f32,
    pub parpadeo: Parpadeo,
    pub capa: usize,
}

pub fn construir() -> (Vec<Pieza>, Vec<Reflector>) {
    let azimut = crate::AZIMUT_INICIAL;
    let mut grillas = Vec::new();
    let mut reflectores = Vec::new();

    let blanco = v(PLATAFORMA_CENTRO.0, 12.0, PLATAFORMA_CENTRO.1);
    let postes = [
        (-44.0, 10.0, 11.0, 2.5, 1.0),
        (44.0, 10.0, 8.0, 2.0, 3.0),
        (-26.0, -32.0, 9.5, 3.0, 2.0),
        (26.0, -32.0, 13.0, 2.0, 5.0),
    ];
    for (i, (derecha, hacia_camara, periodo, apagado, fase)) in postes.into_iter().enumerate() {
        let capa = 1 + i;
        let (grilla, lente, eje) = poste(alrededor(derecha, hacia_camara, PISO), blanco);
        grillas.push(Pieza::reflector(grilla, capa));
        reflectores.push(Reflector {
            posicion: a_mundo(lente),
            eje,
            intensidad: INTENSIDAD_REFLECTOR,
            parpadeo: Parpadeo { periodo, apagado, fase },
            capa,
        });
    }

    let cajas = [
        (-36.0, -14.0, 30.0, 0.3, 4.0, 0.0),
        (36.0, -28.0, 36.0, -0.4, 4.6, 2.1),
        (12.0, -44.0, 44.0, 0.8, 5.3, 4.2),
    ];
    for (derecha, hacia_camara, y, angulo, periodo, fase) in cajas {
        let base = alrededor(derecha, hacia_camara, y);
        let vaiven = Vaiven {
            pivote: a_mundo(base + v(0.0, ALTO_CAJA + 9.0, 0.0)),
            altura: largo_a_mundo(0.9),
            inclinacion: 0.07,
            giro: 0.25,
            periodo,
            fase,
        };
        grillas.push(Pieza::flotando(suministro(base, azimut + angulo), vaiven));
    }

    grillas.push(Pieza::fija(vitrina(alrededor(-32.0, 16.0, PISO), azimut + 0.5)));
    grillas.push(Pieza::fija(construccion(alrededor(38.0, -20.0, PISO), azimut - 0.7)));
    (grillas, reflectores)
}

fn poste(pie: Vec3, blanco: Vec3) -> (Diorama, Vec3, Vec3) {
    let alto = 31.0;
    let tope = pie + v(0.0, alto, 0.0);
    let cabeza = tope + v(0.0, 2.6, 0.0);
    let eje = (blanco - cabeza).normalizado();
    let costado = v(eje.z, 0.0, -eje.x).normalizado();
    let mut m = Modelo::nuevo(pie - v(5.0, 0.5, 5.0), cabeza + v(5.0, 4.0, 5.0));

    m.caja(Op::Sumar(NEGRO), pie - v(2.4, 0.0, 2.4), pie + v(2.4, 1.2, 2.4), 0.4);
    m.cilindro(Op::Sumar(CROMO), pie + v(0.0, 1.5, 0.0), 1, 1.3, 0.8, 0.2);
    m.cilindro(Op::Sumar(CROMO), pie + v(0.0, (1.2 + alto) * 0.5, 0.0), 1, 0.55, alto - 1.2, 0.0);
    m.toro(Op::Sumar(NEGRO), pie + v(0.0, alto * 0.55, 0.0), 1, 0.6, 0.25);

    let ancho = 2.5;
    m.barra(Op::Sumar(NEGRO), tope - costado * ancho, tope + costado * ancho, 0.35, 0.1);
    for lado in [-1.0, 1.0] {
        m.barra(Op::Sumar(NEGRO), tope + costado * (lado * ancho), cabeza + costado * (lado * ancho), 0.35, 0.1);
        m.barra(Op::Sumar(CROMO), cabeza + costado * (lado * 1.7), cabeza + costado * (lado * (ancho + 0.3)), 0.45, 0.1);
    }

    m.barra(Op::Sumar(NEGRO), cabeza - eje * 1.8, cabeza + eje * 1.6, 1.9, 0.5);
    let frente = cabeza + eje * 1.6;
    let alcance = v(2.6, 2.6, 2.6);
    m.aplicar(Op::Sumar(CROMO), frente - alcance, frente + alcance, |p| {
        let aro = barra(p, frente - eje * 0.35, frente + eje * 0.3, 2.0, 0.2);
        aro.max(-barra(p, frente - eje * 1.0, frente + eje * 1.0, 1.55, 0.0))
    });
    m.barra(Op::Sumar(AMARILLO), frente - eje * 0.3, frente + eje * 0.15, 1.5, 0.15);

    (m.terminar(), frente + eje * 1.0, eje)
}

const ALTO_CAJA: f32 = 4.4;

fn suministro(base: Vec3, angulo: f32) -> Diorama {
    let marco = Marco::nuevo(base, angulo);
    let mut m = Modelo::nuevo(base - v(6.0, 1.0, 6.0), base + v(6.0, 19.0, 6.0));
    let (lado, alto) = (2.4, ALTO_CAJA);

    m.caja_en(Op::Sumar(AZUL), marco, v(-lado, 0.0, -lado), v(lado, alto, lado), 0.35);
    let alcance = v(4.0, 0.0, 4.0);
    m.aplicar(Op::Pintar(NEGRO), base - alcance, base + alcance + v(0.0, alto, 0.0), |p| {
        (marco.local(p).y - alto * 0.5).abs() - 0.4
    });
    let (a, y0, y1) = (lado - 0.2, 0.2, alto - 0.2);
    let esquinas = [(-a, -a), (a, -a), (a, a), (-a, a)];
    for i in 0..4 {
        let (x, z) = esquinas[i];
        let (sx, sz) = esquinas[(i + 1) % 4];
        m.capsula(Op::Sumar(CROMO), marco.escena(v(x, y0, z)), marco.escena(v(x, y1, z)), 0.38);
        for y in [y0, y1] {
            m.capsula(Op::Sumar(CROMO), marco.escena(v(x, y, z)), marco.escena(v(sx, y, sz)), 0.38);
        }
    }

    let cima = base + v(0.0, alto + 9.0, 0.0);
    let boca = base + v(0.0, alto + 4.6, 0.0);
    m.gota(Op::Sumar(AZUL), cima, boca, 4.2, 1.0);
    for k in 0..2 {
        let (sen, cos) = (angulo + k as f32 * std::f32::consts::FRAC_PI_2).sin_cos();
        m.aplicar(Op::Pintar(NEGRO), cima - v(4.5, 5.0, 4.5), cima + v(4.5, 4.5, 4.5), |p| {
            let costura = ((p.x - base.x) * cos - (p.z - base.z) * sen).abs() - 0.16;
            costura.max(boca.y + 0.8 - p.y)
        });
    }
    m.toro(Op::Sumar(CROMO), base + v(0.0, alto + 3.6, 0.0), 1, 0.95, 0.25);
    for (x, z) in esquinas {
        let arriba = marco.escena(v(x * 0.27, alto + 3.5, z * 0.27));
        m.capsula(Op::Sumar(NEGRO), marco.escena(v(x * 0.85, alto, z * 0.85)), arriba, 0.22);
    }
    m.terminar()
}

fn vitrina(pie: Vec3, angulo: f32) -> Diorama {
    let marco = Marco::nuevo(pie, angulo);
    let e = 1.3;
    let q = |x: f32, y: f32, z: f32| v(x, y, z) * e;
    let mut m = Modelo::nuevo(pie - q(5.5, 0.5, 5.5), pie + q(5.5, 13.5, 5.5));

    m.caja_en(Op::Sumar(VIDRIO), marco, q(-3.5, 5.0, -3.5), q(3.5, 12.4, 3.5), 0.4);
    m.caja_en(Op::Restar, marco, q(-3.5, 4.0, -3.5) + v(0.5, 0.0, 0.5), q(3.5, 12.4, 3.5) - v(0.5, 0.5, 0.5), 0.2);
    m.caja_en(Op::Sumar(CROMO), marco, q(-4.2, 0.0, -4.2), q(4.2, 0.8, 4.2), 0.3);
    m.caja_en(Op::Sumar(NEGRO), marco, q(-3.6, 0.6, -3.6), q(3.6, 4.8, 3.6), 0.3);
    m.caja_en(Op::Sumar(CROMO), marco, q(-4.0, 4.6, -4.0), q(4.0, 5.6, 4.0), 0.25);
    for (sx, sz) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let (x, z) = (3.35 * sx, 3.35 * sz);
        m.capsula(Op::Sumar(CROMO), marco.escena(q(x, 5.6, z)), marco.escena(q(x, 12.2, z)), 0.4);
    }
    m.cofre(marco.escena(q(0.0, 5.6, 0.0)), angulo, 1.1 * e);
    m.terminar()
}

fn construccion(pie: Vec3, angulo: f32) -> Diorama {
    let marco = Marco::nuevo(pie, angulo);
    let l = 11.0;
    let (desde, hasta) = marco.envolvente(v(-l - 1.5, -0.5, -1.5), v(l + 1.5, l + 1.5, l + 1.5));
    let mut m = Modelo::nuevo(desde, hasta);

    let (pared_min, pared_max) = (v(-l - 0.5, 0.0, 0.0), v(-0.5, l, 0.8));
    m.caja_en(Op::Sumar(CROMO), marco, pared_min, pared_max, 0.15);
    let mut x = pared_min.x + 1.6;
    while x < pared_max.x - 1.2 {
        m.caja_en(Op::Restar, marco, v(x - 0.22, 1.0, 0.55), v(x + 0.22, l - 1.0, 1.5), 0.0);
        m.caja_en(Op::Restar, marco, v(x - 0.22, 1.0, -0.7), v(x + 0.22, l - 1.0, 0.25), 0.0);
        x += 1.1;
    }
    let (marco_min, marco_max) = (pared_min - v(0.0, 0.0, 0.3), pared_max + v(0.0, 0.0, 0.3));
    let (caja_min, caja_max) = marco.envolvente(marco_min, marco_max);
    m.aplicar(Op::Sumar(NEGRO), caja_min, caja_max, |p| {
        let q = marco.local(p);
        let borde = caja_redondeada(q, marco_min, marco_max, 0.15);
        let hueco = caja_redondeada(q, pared_min + v(0.9, 0.9, -1.0), pared_max - v(0.9, 0.9, -1.0), 0.0);
        borde.max(-hueco)
    });
    let (x0, x1, y0, y1) = (pared_min.x + 0.9, pared_max.x - 0.9, 0.9, l - 0.9);
    for z in [1.1, -0.3] {
        m.barra(Op::Sumar(NEGRO), marco.escena(v(x0, y0, z)), marco.escena(v(x1, y1, z)), 0.32, 0.1);
        m.barra(Op::Sumar(NEGRO), marco.escena(v(x0, y1, z)), marco.escena(v(x1, y0, z)), 0.32, 0.1);
    }

    let raiz = std::f32::consts::FRAC_1_SQRT_2;
    let (pendiente, normal) = (v(0.0, raiz, -raiz), v(0.0, raiz, raiz));
    let largo = l * std::f32::consts::SQRT_2;
    let centro = v(l * 0.5 + 0.5, l * 0.5, l * 0.5);
    let en_rampa = move |p: Vec3| {
        let q = marco.local(p) - centro;
        v(q.x, q.dot(pendiente), q.dot(normal))
    };
    let (rampa_min, rampa_max) = marco.envolvente(v(0.2, -0.5, -1.0), v(l + 0.8, l + 1.0, l + 1.0));
    let (ancho, medio) = (l * 0.5, largo * 0.5);
    let pieza = move |p: Vec3, min: Vec3, max: Vec3, radio: f32| {
        caja_redondeada(en_rampa(p), min, max, radio).max(-marco.local(p).y)
    };
    m.aplicar(Op::Sumar(CROMO), rampa_min, rampa_max, |p| {
        pieza(p, v(-ancho, -medio, -0.7), v(ancho, medio, 0.0), 0.15)
    });
    for lado in [-1.0, 1.0] {
        let (a, b) = (lado * (ancho - 0.8), lado * ancho);
        m.aplicar(Op::Sumar(NEGRO), rampa_min, rampa_max, |p| {
            pieza(p, v(a.min(b), -medio, -0.9), v(a.max(b), medio, 0.35), 0.15)
        });
    }
    let mut a = -medio + 1.4;
    while a < medio - 0.7 {
        m.aplicar(Op::Sumar(NEGRO), rampa_min, rampa_max, |p| {
            pieza(p, v(-ancho + 0.8, a - 0.25, -0.2), v(ancho - 0.8, a + 0.25, 0.3), 0.1)
        });
        a += 1.5;
    }
    m.terminar()
}
