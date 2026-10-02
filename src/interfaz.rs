use crate::framebuffer::{Color, FrameBuffer};
use crate::musica::Musica;
use std::time::{Duration, Instant};

const SALTO: f32 = 10.0;
const VOLUMEN_POR_RUEDA: f32 = 0.05;
const VISIBLE_SIN_USO: Duration = Duration::from_secs(3);

const MARGEN: usize = 16;
const ANCHO: usize = 292;
const ALTO: usize = 72;
const LADO_ICONO: usize = 44;

const FONDO: Color = Color { r: 12, g: 14, b: 40 };
const BORDE: Color = Color { r: 90, g: 100, b: 200 };
const ICONO: Color = Color { r: 235, g: 238, b: 255 };
const RESALTE: Color = Color { r: 255, g: 214, b: 64 };
const PISTA: Color = Color { r: 70, g: 80, b: 140 };
const TEXTO: Color = Color { r: 210, g: 215, b: 245 };

#[derive(Clone, Copy, PartialEq)]
enum Control {
    Reiniciar,
    PausarSeguir,
    Adelantar,
    Volumen,
    Avance,
    Fondo,
}

#[derive(Clone, Copy)]
struct Caja {
    x: usize,
    y: usize,
    ancho: usize,
    alto: usize,
}

impl Caja {
    fn contiene(&self, (x, y): (f32, f32)) -> bool {
        x >= self.x as f32 && y >= self.y as f32 && x < (self.x + self.ancho) as f32 && y < (self.y + self.alto) as f32
    }

    fn fraccion(&self, x: f32) -> f32 {
        ((x - self.x as f32) / self.ancho as f32).clamp(0.0, 1.0)
    }
}

pub struct Panel {
    agarrado: Option<Control>,
    presionado_antes: bool,
    avance_pedido: Option<f32>,
    abierto: bool,
    ultimo_uso: Instant,
}

impl Panel {
    pub fn nuevo() -> Self {
        Panel {
            agarrado: None,
            presionado_antes: false,
            avance_pedido: None,
            abierto: false,
            ultimo_uso: Instant::now(),
        }
    }

    fn icono(ancho_ventana: usize, alto_ventana: usize) -> Caja {
        Caja {
            x: ancho_ventana.saturating_sub(LADO_ICONO + MARGEN),
            y: alto_ventana.saturating_sub(LADO_ICONO + MARGEN),
            ancho: LADO_ICONO,
            alto: LADO_ICONO,
        }
    }

    fn cajas(ancho_ventana: usize, alto_ventana: usize) -> (Caja, [(Control, Caja); 5]) {
        let x = ancho_ventana.saturating_sub(ANCHO + MARGEN);
        let y = alto_ventana.saturating_sub(ALTO + MARGEN);
        let caja = |dx: usize, dy: usize, ancho: usize, alto: usize| Caja { x: x + dx, y: y + dy, ancho, alto };
        (
            caja(0, 0, ANCHO, ALTO),
            [
                (Control::Reiniciar, caja(10, 8, 32, 32)),
                (Control::PausarSeguir, caja(46, 8, 32, 32)),
                (Control::Adelantar, caja(82, 8, 32, 32)),
                (Control::Volumen, caja(150, 12, 132, 24)),
                (Control::Avance, caja(10, 46, 176, 18)),
            ],
        )
    }

    fn control_en(mouse: (f32, f32), ancho: usize, alto: usize) -> Option<Control> {
        let (panel, controles) = Self::cajas(ancho, alto);
        if !panel.contiene(mouse) {
            return None;
        }
        Some(controles.iter().find(|(_, caja)| caja.contiene(mouse)).map_or(Control::Fondo, |&(c, _)| c))
    }

    pub fn atender(
        &mut self,
        musica: &Musica,
        mouse: Option<(f32, f32)>,
        presionado: bool,
        rueda: Option<f32>,
        ancho: usize,
        alto: usize,
    ) -> bool {
        let recien = presionado && !self.presionado_antes;
        self.presionado_antes = presionado;

        if !self.abierto {
            let sobre_icono = mouse.is_some_and(|m| Self::icono(ancho, alto).contiene(m));
            if !sobre_icono || !(recien || rueda.is_some()) {
                return false;
            }
            self.abierto = true;
            self.ultimo_uso = Instant::now();
            if let Some(pasos) = rueda {
                musica.fijar_volumen(musica.volumen() + pasos * VOLUMEN_POR_RUEDA);
            }
            self.agarrado = recien.then_some(Control::Fondo);
            return true;
        }

        let encima = mouse.and_then(|m| Self::control_en(m, ancho, alto));

        if recien {
            self.agarrado = encima;
            match encima {
                Some(Control::Reiniciar) => musica.reiniciar(),
                Some(Control::PausarSeguir) => musica.alternar_pausa(),
                Some(Control::Adelantar) => musica.adelantar(SALTO),
                _ => {}
            }
        } else if !presionado {
            self.agarrado = None;
            self.avance_pedido = None;
        }

        if let (Some(control), Some((x, _))) = (self.agarrado, mouse) {
            let (_, controles) = Self::cajas(ancho, alto);
            let caja = |buscado: Control| controles.iter().find(|(c, _)| *c == buscado).unwrap().1;
            match control {
                Control::Volumen => musica.fijar_volumen(caja(Control::Volumen).fraccion(x)),
                Control::Avance => {
                    let fraccion = caja(Control::Avance).fraccion(x);
                    let movida = self.avance_pedido.is_none_or(|antes| (fraccion - antes).abs() > 0.01);
                    if let (true, Some(total)) = (movida, musica.duracion()) {
                        musica.ir_a(total.mul_f32(fraccion));
                        self.avance_pedido = Some(fraccion);
                    }
                }
                _ => {}
            }
        }

        let rueda_en_panel = encima.is_some() && rueda.is_some();
        if let (true, Some(pasos)) = (rueda_en_panel, rueda) {
            musica.fijar_volumen(musica.volumen() + pasos * VOLUMEN_POR_RUEDA);
        }

        if self.agarrado.is_some() || encima.is_some() {
            self.ultimo_uso = Instant::now();
        } else if self.ultimo_uso.elapsed() >= VISIBLE_SIN_USO {
            self.abierto = false;
        }
        self.agarrado.is_some() || rueda_en_panel
    }

    pub fn dibujar(&self, fb: &mut FrameBuffer, musica: &Musica, mouse: Option<(f32, f32)>) {
        let (ancho, alto) = fb.dimensions();
        if !self.abierto {
            let icono = Self::icono(ancho, alto);
            let encima = mouse.is_some_and(|m| icono.contiene(m));
            fondo(fb, icono);
            if encima {
                rectangulo(fb, icono, ICONO, 0.12);
            }
            let color = if encima { RESALTE } else { ICONO };
            let (x, y) = (icono.x + 9, icono.y + 12);
            parlante(fb, x, y, 2, if musica.pausada() { 0.0 } else { musica.volumen() }, color);
            if musica.pausada() {
                rectangulo(fb, Caja { x: x + 20, y: y + 4, ancho: 3, alto: 12 }, color, 1.0);
                rectangulo(fb, Caja { x: x + 26, y: y + 4, ancho: 3, alto: 12 }, color, 1.0);
            }
            return;
        }

        let (panel, controles) = Self::cajas(ancho, alto);
        let encima = mouse.and_then(|m| Self::control_en(m, ancho, alto));
        let resaltado = |control: Control| encima == Some(control) || self.agarrado == Some(control);
        fondo(fb, panel);

        for &(control, caja) in &controles {
            let color = if resaltado(control) { RESALTE } else { ICONO };
            let (x, y) = (caja.x as f32, caja.y as f32);
            if matches!(control, Control::Reiniciar | Control::PausarSeguir | Control::Adelantar) && resaltado(control) {
                rectangulo(fb, caja, ICONO, 0.12);
            }
            match control {
                Control::Reiniciar => {
                    rectangulo(fb, Caja { x: caja.x + 8, y: caja.y + 9, ancho: 3, alto: 14 }, color, 1.0);
                    triangulo(fb, (x + 24.0, y + 9.0), (x + 24.0, y + 23.0), (x + 12.0, y + 16.0), color);
                }
                Control::PausarSeguir => {
                    if musica.pausada() {
                        triangulo(fb, (x + 11.0, y + 8.0), (x + 11.0, y + 24.0), (x + 24.0, y + 16.0), color);
                    } else {
                        rectangulo(fb, Caja { x: caja.x + 10, y: caja.y + 8, ancho: 4, alto: 16 }, color, 1.0);
                        rectangulo(fb, Caja { x: caja.x + 18, y: caja.y + 8, ancho: 4, alto: 16 }, color, 1.0);
                    }
                }
                Control::Adelantar => {
                    triangulo(fb, (x + 7.0, y + 9.0), (x + 7.0, y + 23.0), (x + 16.0, y + 16.0), color);
                    triangulo(fb, (x + 16.0, y + 9.0), (x + 16.0, y + 23.0), (x + 25.0, y + 16.0), color);
                }
                Control::Volumen => {
                    parlante(fb, caja.x - 22, caja.y + 6, 1, musica.volumen(), ICONO);
                    barra(fb, caja, musica.volumen(), color);
                }
                Control::Avance => {
                    let avance = match musica.duracion() {
                        Some(total) if !total.is_zero() => (musica.posicion().as_secs_f32() / total.as_secs_f32()).min(1.0),
                        _ => 0.0,
                    };
                    barra(fb, caja, avance, color);
                }
                Control::Fondo => {}
            }
        }

        let total = musica.duracion().map_or(String::from("-:--"), |t| minutos(t + Duration::from_millis(500)));
        let tiempo = format!("{} / {}", minutos(musica.posicion()), total);
        texto(fb, panel.x + 194, panel.y + 50, &tiempo, TEXTO);
    }
}

fn fondo(fb: &mut FrameBuffer, caja: Caja) {
    rectangulo(fb, caja, FONDO, 0.62);
    rectangulo(fb, Caja { alto: 1, ..caja }, BORDE, 0.8);
    rectangulo(fb, Caja { y: caja.y + caja.alto - 1, alto: 1, ..caja }, BORDE, 0.8);
    rectangulo(fb, Caja { ancho: 1, ..caja }, BORDE, 0.8);
    rectangulo(fb, Caja { x: caja.x + caja.ancho - 1, ancho: 1, ..caja }, BORDE, 0.8);
}

fn parlante(fb: &mut FrameBuffer, x: usize, y: usize, escala: usize, volumen: f32, color: Color) {
    let e = escala;
    let punto = |dx: usize, dy: usize| ((x + dx * e) as f32, (y + dy * e) as f32);
    rectangulo(fb, Caja { x, y: y + 4 * e, ancho: 4 * e, alto: 6 * e }, color, 1.0);
    triangulo(fb, punto(4, 4), punto(4, 10), punto(9, 0), color);
    triangulo(fb, punto(4, 10), punto(9, 14), punto(9, 0), color);
    for onda in 0..3 {
        if volumen > onda as f32 / 3.0 {
            let alto = (4 + 3 * onda) * e;
            rectangulo(fb, Caja { x: x + (11 + 3 * onda) * e, y: y + 7 * e - alto / 2, ancho: e, alto }, color, 1.0);
        }
    }
}

fn minutos(t: Duration) -> String {
    let s = t.as_secs();
    format!("{}:{:02}", s / 60, s % 60)
}

fn rectangulo(fb: &mut FrameBuffer, caja: Caja, color: Color, alfa: f32) {
    for y in caja.y..caja.y + caja.alto {
        for x in caja.x..caja.x + caja.ancho {
            fb.point_translucido(x, y, color, alfa);
        }
    }
}

fn triangulo(fb: &mut FrameBuffer, a: (f32, f32), b: (f32, f32), c: (f32, f32), color: Color) {
    let lado = |p: (f32, f32), q: (f32, f32), r: (f32, f32)| (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0);
    let area = lado(a, b, c);
    let (x0, x1) = (a.0.min(b.0).min(c.0).floor() as usize, a.0.max(b.0).max(c.0).ceil() as usize);
    let (y0, y1) = (a.1.min(b.1).min(c.1).floor() as usize, a.1.max(b.1).max(c.1).ceil() as usize);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            let (u, v, w) = (lado(b, c, p), lado(c, a, p), lado(a, b, p));
            if u * area >= 0.0 && v * area >= 0.0 && w * area >= 0.0 {
                fb.point(x, y, color);
            }
        }
    }
}

fn barra(fb: &mut FrameBuffer, caja: Caja, fraccion: f32, color: Color) {
    let medio = caja.y + caja.alto / 2;
    let lleno = (fraccion * caja.ancho as f32) as usize;
    rectangulo(fb, Caja { x: caja.x, y: medio - 2, ancho: caja.ancho, alto: 4 }, PISTA, 1.0);
    rectangulo(fb, Caja { x: caja.x, y: medio - 2, ancho: lleno, alto: 4 }, color, 1.0);
    let (cx, cy, radio) = ((caja.x + lleno) as f32, medio as f32, 6.0);
    for y in (cy - radio) as usize..=(cy + radio) as usize {
        for x in (cx - radio).max(0.0) as usize..=(cx + radio) as usize {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            if dx * dx + dy * dy <= radio * radio {
                fb.point(x, y, color);
            }
        }
    }
}

fn letra(c: char) -> [u8; 5] {
    match c {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b001, 0b001, 0b001],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        _ => [0; 5],
    }
}

fn texto(fb: &mut FrameBuffer, x: usize, y: usize, s: &str, color: Color) {
    const ESCALA: usize = 2;
    for (i, c) in s.chars().enumerate() {
        let x0 = x + i * 4 * ESCALA;
        for (fila, bits) in letra(c).iter().enumerate() {
            for columna in 0..3 {
                if bits & (0b100 >> columna) != 0 {
                    rectangulo(fb, Caja { x: x0 + columna * ESCALA, y: y + fila * ESCALA, ancho: ESCALA, alto: ESCALA }, color, 1.0);
                }
            }
        }
    }
}
