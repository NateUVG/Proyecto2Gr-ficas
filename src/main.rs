mod animacion;
mod bus;
mod camara;
mod capas;
mod decorado;
mod diorama;
mod formas;
mod framebuffer;
mod interfaz;
mod material;
mod modelo;
mod musica;
mod ray_intersect;
mod skybox;
mod textura;
mod vector;

use animacion::{Parpadeo, Pieza};
use camara::Camara;
use capas::{CAPAS, Capas, Imagen, Pixel};
use framebuffer::{Color, FrameBuffer};
use interfaz::Panel;
use material::{AMARILLO, Material};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, ScaleMode, Window, WindowOptions};
use musica::Musica;
use ray_intersect::Intersect;
use skybox::{color_lobby, color_piso, transicion};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use vector::Vec3;

const ANCHO: usize = 800;
const ALTO: usize = 600;
const FOV_GRADOS: f32 = 40.0;
const RAYOS_PREVIA: usize = 120_000;
const RAYOS_FINAL: usize = 2_000_000;
const PROFUNDIDAD_MAX: u32 = 4;
const BRILLO_ESPECULAR: f32 = 40.0;
const LUZ_AMBIENTE: f32 = 0.30;
const EPSILON: f32 = 1e-4;
const SEPARACION: f32 = 0.05;

const AZIMUT_INICIAL: f32 = 0.6;

const VELOCIDAD_ROTACION: f32 = 0.04;
const VELOCIDAD_ZOOM: f32 = 0.12;
const SENSIBILIDAD_MOUSE: f32 = 0.008;
const VELOCIDAD_ZOOM_RUEDA: f32 = 0.3;
const PASO_MAXIMO: f32 = 0.1;
const ESPERA_FINAL: Duration = Duration::from_millis(150);
const CANCION: &str = "assets/LobbyClasic1.mp3";
const VOLUMEN_INICIAL: f32 = 0.6;
const BRILLO_LENTE: Vec3 = Vec3 { x: 1.0, y: 0.85, z: 0.45 };
const COLOR_HAZ: Vec3 = Vec3 { x: 1.0, y: 0.92, z: 0.75 };
const DENSIDAD_HAZ: f32 = 0.3;
const ALCANCE_HAZ: f32 = 6.0;
const MUESTRAS_HAZ: usize = 8;

struct Luz {
    posicion: Vec3,
    intensidad: f32,
    sombras: bool,
    capa: usize,
    parpadeo: Option<Parpadeo>,
    foco: Option<Foco>,
}

#[derive(Clone, Copy)]
struct Foco {
    eje: Vec3,
    cos_centro: f32,
    cos_borde: f32,
}

fn haz(luz: &Luz, foco: &Foco, origen: Vec3, direccion: Vec3, hasta: f32) -> Vec3 {
    let w = origen - luz.posicion;
    let (de, we) = (direccion.dot(foco.eje), w.dot(foco.eje));
    let c2 = foco.cos_borde * foco.cos_borde;
    let a = de * de - c2;
    let b = 2.0 * (de * we - c2 * direccion.dot(w));
    let c = we * we - c2 * w.dot(w);
    if a.abs() < 1e-6 {
        return Vec3::new(0.0, 0.0, 0.0);
    }
    let discriminante = b * b - 4.0 * a * c;
    let tramos: [(f32, f32); 2] = if discriminante < 0.0 {
        if a < 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        [(f32::NEG_INFINITY, f32::INFINITY), (0.0, -1.0)]
    } else {
        let raiz = discriminante.sqrt();
        let (t1, t2) = ((-b - raiz) / (2.0 * a), (-b + raiz) / (2.0 * a));
        let (t1, t2) = (t1.min(t2), t1.max(t2));
        if a < 0.0 { [(t1, t2), (0.0, -1.0)] } else { [(f32::NEG_INFINITY, t1), (t2, f32::INFINITY)] }
    };
    let largo = 4.0 * ALCANCE_HAZ;
    let (mut desde, mut limite) = (0.0f32, hasta);
    if de.abs() > 1e-6 {
        let (ta, tb) = (-we / de, (largo - we) / de);
        desde = desde.max(ta.min(tb));
        limite = limite.min(ta.max(tb));
    } else if we < 0.0 || we > largo {
        return Vec3::new(0.0, 0.0, 0.0);
    }

    let mut brillo = 0.0;
    for (t0, t1) in tramos {
        let (t0, t1) = (t0.max(desde), t1.min(limite));
        if t1 <= t0 {
            continue;
        }
        let paso = (t1 - t0) / MUESTRAS_HAZ as f32;
        for k in 0..MUESTRAS_HAZ {
            let p = w + direccion * (t0 + (k as f32 + 0.5) * paso);
            let a_lo_largo = p.dot(foco.eje);
            let perfil = transicion(foco.cos_borde, foco.cos_centro, a_lo_largo / p.length());
            let distancia = a_lo_largo / ALCANCE_HAZ;
            brillo += perfil / (1.0 + distancia * distancia) * paso;
        }
    }
    COLOR_HAZ * (brillo * DENSIDAD_HAZ * luz.intensidad)
}

impl Luz {
    fn intensidad_hacia(&self, punto: Vec3) -> f32 {
        match self.foco {
            None => self.intensidad,
            Some(foco) => {
                let coseno = (punto - self.posicion).normalizado().dot(foco.eje);
                self.intensidad * transicion(foco.cos_borde, foco.cos_centro, coseno)
            }
        }
    }
}

struct Escena {
    piezas: Vec<Pieza>,
    materiales: Vec<Material>,
    luces: Vec<Luz>,
    piso: f32,
}

const MAX_GRILLAS: usize = 32;

impl Escena {
    fn animar(&mut self, t: f32) {
        for pieza in &mut self.piezas {
            pieza.animar(t);
        }
    }

    fn encendido(&self, t: f32) -> [f32; CAPAS] {
        let mut encendido = [1.0; CAPAS];
        for luz in &self.luces {
            if let Some(parpadeo) = luz.parpadeo {
                encendido[luz.capa] = parpadeo.encendido(t);
            }
        }
        encendido
    }

    fn chocar(&self, origen: Vec3, direccion: Vec3) -> Intersect {
        let mut entradas = [(0.0f32, 0usize); MAX_GRILLAS];
        let mut n = 0;
        for (i, pieza) in self.piezas.iter().enumerate() {
            if let Some(t) = pieza.entrada(origen, direccion) {
                let mut j = n;
                while j > 0 && entradas[j - 1].0 > t {
                    entradas[j] = entradas[j - 1];
                    j -= 1;
                }
                entradas[j] = (t, i);
                n += 1;
            }
        }
        let mut mejor = Intersect::vacio();
        for &(t, i) in &entradas[..n] {
            if t > mejor.distancia {
                break;
            }
            let hit = self.piezas[i].chocar(origen, direccion);
            if hit.is_intersecting && hit.distancia < mejor.distancia {
                mejor = Intersect { pieza: i, ..hit };
            }
        }
        mejor
    }
}

fn visibilidad_luz(escena: &Escena, luz: &Luz, punto: Vec3, normal_cara: Vec3, normal_suave: Vec3) -> f32 {
    if !luz.sombras {
        return 1.0;
    }
    let origen = punto + normal_cara * EPSILON + normal_suave * SEPARACION;
    let hacia = luz.posicion - origen;
    let distancia = hacia.length();
    let hit = escena.chocar(origen, hacia * (1.0 / distancia));
    if !hit.is_intersecting || hit.distancia > distancia {
        return 1.0;
    }
    escena.materiales[hit.material].transparencia * 0.8
}

fn fondo(escena: &Escena, origen: Vec3, direccion: Vec3, profundidad: u32) -> Capas {
    if direccion.y < -1e-4 {
        let t = (escena.piso - origen.y) / direccion.y;
        if t > 0.0 {
            return color_del_piso(escena, origen + direccion * t, direccion, profundidad);
        }
    }
    Capas::en(0, color_lobby(direccion))
}

fn color_del_piso(escena: &Escena, punto: Vec3, direccion: Vec3, profundidad: u32) -> Capas {
    let arriba = Vec3::new(0.0, 1.0, 0.0);
    let piso = color_piso(punto);
    let mut local = Capas::en(0, piso * 0.45);
    for luz in &escena.luces {
        let incidencia = arriba.dot((luz.posicion - punto).normalizado());
        let intensidad = luz.intensidad_hacia(punto);
        if incidencia > 0.0 && intensidad > 0.0 {
            let visible = visibilidad_luz(escena, luz, punto, arriba, arriba);
            let directa = incidencia * intensidad * (0.35 + 0.65 * visible);
            local.0[luz.capa] = local.0[luz.capa] + piso * (0.55 * directa);
        }
    }

    let reflejada = direccion.reflejar(arriba);
    let reflejo = if profundidad > 0 {
        cast_ray(escena, punto + arriba * EPSILON, reflejada, profundidad - 1)
    } else {
        Capas::en(0, color_lobby(reflejada))
    };
    let lejania = Vec3::new(punto.x, 0.0, punto.z).length();
    let brillo = (0.5 + 0.3 * (1.0 + direccion.y).powi(4)) * (1.0 - 0.6 * transicion(3.0, 10.0, lejania));
    let color = capas::mezclar(local, reflejo, brillo);

    capas::mezclar(color, Capas::en(0, color_lobby(direccion)), transicion(7.0, 18.0, lejania))
}

fn cast_ray(escena: &Escena, origen: Vec3, direccion: Vec3, profundidad: u32) -> Capas {
    trazar(escena, origen, direccion, profundidad).0
}

fn trazar(escena: &Escena, origen: Vec3, direccion: Vec3, profundidad: u32) -> (Capas, bool) {
    let hit = escena.chocar(origen, direccion);
    let al_piso = !hit.is_intersecting && direccion.y < -1e-4 && (escena.piso - origen.y) / direccion.y > 0.0;
    let (mut color, hasta) = if hit.is_intersecting {
        (sombrear(escena, &hit, direccion, profundidad), hit.distancia)
    } else if al_piso {
        (fondo(escena, origen, direccion, profundidad), (escena.piso - origen.y) / direccion.y)
    } else {
        (fondo(escena, origen, direccion, profundidad), f32::INFINITY)
    };
    for luz in &escena.luces {
        if let Some(foco) = &luz.foco {
            color.0[luz.capa] = color.0[luz.capa] + haz(luz, foco, origen, direccion, hasta);
        }
    }
    (color, al_piso)
}

fn sombrear(escena: &Escena, hit: &Intersect, direccion: Vec3, profundidad: u32) -> Capas {
    let material = &escena.materiales[hit.material];

    let entrando = direccion.dot(hit.normal) < 0.0;
    let (normal_cara, normal) = if entrando {
        (hit.normal, hit.normal_suave)
    } else {
        (-hit.normal, -hit.normal_suave)
    };

    let kr = material.reflectividad;
    let kt = if entrando { material.transparencia } else { 1.0 - kr };
    let peso_local = (1.0 - kr - kt).max(0.0);

    let mut color = Capas::cero();
    if peso_local > 0.0 {
        let base = material.textura.muestrear(hit.uv.0, hit.uv.1);
        let mut local = Capas::en(0, base * LUZ_AMBIENTE);
        for luz in &escena.luces {
            let hacia_luz = (luz.posicion - hit.punto).normalizado();
            let incidencia = normal.dot(hacia_luz);
            let intensidad = luz.intensidad_hacia(hit.punto);
            if incidencia <= 0.0 || intensidad <= 0.0 {
                continue;
            }
            let visible = visibilidad_luz(escena, luz, hit.punto, normal_cara, normal);
            let difuso = incidencia * intensidad * visible;
            let especular = (-hacia_luz).reflejar(normal).dot(-direccion).max(0.0).powf(BRILLO_ESPECULAR) * intensidad * visible;
            let aporte = base * (difuso * material.albedo) + Vec3::new(1.0, 1.0, 1.0) * (especular * material.especular);
            local.0[luz.capa] = local.0[luz.capa] + aporte;
        }
        if hit.material == AMARILLO {
            if let Some(capa) = escena.piezas[hit.pieza].lente() {
                local.0[capa] = local.0[capa] + BRILLO_LENTE;
            }
        }
        color = local * peso_local;
    }

    let mut reflejada = direccion.reflejar(normal);
    if reflejada.dot(normal_cara) <= 0.0 {
        reflejada = direccion.reflejar(normal_cara);
    }
    if kr > 0.0 {
        let c = if profundidad > 0 {
            let separacion = if entrando { normal * SEPARACION } else { Vec3::new(0.0, 0.0, 0.0) };
            cast_ray(escena, hit.punto + normal_cara * EPSILON + separacion, reflejada, profundidad - 1)
        } else {
            fondo(escena, hit.punto, reflejada, 0)
        };
        color = color + c * kr;
    }
    if kt > 0.0 {
        let c = if profundidad > 0 {
            let eta = if entrando { 1.0 / material.indice_refraccion } else { material.indice_refraccion };
            match direccion.refractar(normal, eta) {
                Some(refractada) => cast_ray(escena, hit.punto - normal_cara * EPSILON, refractada.normalizado(), profundidad - 1),
                None => cast_ray(escena, hit.punto + normal_cara * EPSILON, reflejada, profundidad - 1),
            }
        } else {
            fondo(escena, hit.punto, direccion, 0)
        };
        color = color + c * kt;
    }
    color
}

#[derive(Clone, Copy)]
enum Calidad<'a> {
    Previa,
    Final,
    Zonas(&'a Zonas),
}

#[derive(Clone, Default)]
struct Zonas {
    piezas: Vec<Zona>,
    piso: Vec<Zona>,
}

impl Zonas {
    fn unir(&self, otras: &Zonas) -> Zonas {
        Zonas {
            piezas: [self.piezas.as_slice(), &otras.piezas].concat(),
            piso: [self.piso.as_slice(), &otras.piso].concat(),
        }
    }
}

#[derive(Clone, Copy)]
struct Zona {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
}

fn marcar(zonas: &Zonas, ve_piso: &[bool], bloque: usize, ancho: usize, alto: usize) -> (Vec<bool>, usize) {
    let mut marcados = vec![false; ancho * alto];
    let mut cantidad = 0;
    let solo_piso = zonas.piso.iter().map(|z| (z, bloque == 1));
    for (z, filtrar) in zonas.piezas.iter().map(|z| (z, false)).chain(solo_piso) {
        let (x0, y0) = (z.x0 / bloque * bloque, z.y0 / bloque * bloque);
        let (x1, y1) = (z.x1.next_multiple_of(bloque).min(ancho), z.y1.next_multiple_of(bloque).min(alto));
        for y in y0..y1 {
            for i in y * ancho + x0..y * ancho + x1 {
                if !marcados[i] && (!filtrar || ve_piso[i]) {
                    marcados[i] = true;
                    cantidad += 1;
                }
            }
        }
    }
    (marcados, cantidad)
}

fn lado_de_bloque(pixeles: usize, rayos: usize) -> usize {
    ((pixeles as f32 / rayos as f32).sqrt().round() as usize).max(1)
}

struct Proyeccion {
    ojo: Vec3,
    adelante: Vec3,
    derecha: Vec3,
    arriba: Vec3,
    escala_x: f32,
    escala_y: f32,
    ancho: usize,
    alto: usize,
}

impl Proyeccion {
    fn nueva(camara: &Camara, ancho: usize, alto: usize) -> Self {
        let (ojo, adelante, derecha, arriba) = camara.base_de_rayos();
        let escala = (FOV_GRADOS.to_radians() / 2.0).tan();
        Proyeccion {
            ojo,
            adelante,
            derecha,
            arriba,
            escala_x: escala * ancho as f32 / alto as f32,
            escala_y: escala,
            ancho,
            alto,
        }
    }

    fn zona(&self, puntos: &[Vec3]) -> Zona {
        let entera = Zona { x0: 0, y0: 0, x1: self.ancho, y1: self.alto };
        let (mut x0, mut y0, mut x1, mut y1) = (f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for &p in puntos {
            let v = p - self.ojo;
            let z = v.dot(self.adelante);
            if z < 1e-3 {
                return entera;
            }
            let x = (v.dot(self.derecha) / (z * self.escala_x) + 1.0) * 0.5 * self.ancho as f32;
            let y = (1.0 - v.dot(self.arriba) / (z * self.escala_y)) * 0.5 * self.alto as f32;
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
        let recortar = |valor: f32, maximo: usize| valor.clamp(0.0, maximo as f32) as usize;
        Zona {
            x0: recortar(x0 - 2.0, self.ancho),
            y0: recortar(y0 - 2.0, self.alto),
            x1: recortar(x1 + 3.0, self.ancho),
            y1: recortar(y1 + 3.0, self.alto),
        }
    }
}

fn zonas_en_movimiento(escena: &Escena, camara: &Camara, ancho: usize, alto: usize) -> Zonas {
    let proyeccion = Proyeccion::nueva(camara, ancho, alto);
    let mut zonas = Zonas::default();
    for esquinas in escena.piezas.iter().filter_map(Pieza::esquinas) {
        zonas.piezas.push(proyeccion.zona(&esquinas));
        let reflejo = esquinas.map(|p| Vec3::new(p.x, 2.0 * escena.piso - p.y, p.z));
        zonas.piso.push(proyeccion.zona(&reflejo));
        for luz in escena.luces.iter().filter(|l| l.sombras) {
            if esquinas.iter().any(|p| p.y >= luz.posicion.y) {
                zonas.piso.push(proyeccion.zona(&[proyeccion.ojo]));
                continue;
            }
            let sombra = esquinas.map(|p| {
                let t = (escena.piso - luz.posicion.y) / (p.y - luz.posicion.y);
                luz.posicion + (p - luz.posicion) * t
            });
            zonas.piso.push(proyeccion.zona(&sombra));
        }
    }
    zonas
}

fn renderizar(imagen: &mut Imagen, escena: &Escena, camara: &Camara, calidad: Calidad) {
    let (ancho, alto) = imagen.dimensiones();
    let (ojo, adelante, derecha, arriba) = camara.base_de_rayos();
    let aspecto = ancho as f32 / alto as f32;
    let escala = (FOV_GRADOS.to_radians() / 2.0).tan();
    let pixeles = ancho * alto;
    let (bloque, muestras_por_lado, marcados) = match calidad {
        Calidad::Previa => (lado_de_bloque(pixeles, RAYOS_PREVIA).max(2), 1, None),
        Calidad::Final => (1, if pixeles * 4 <= RAYOS_FINAL { 2 } else { 1 }, None),
        Calidad::Zonas(zonas) => {
            let (marcados, cantidad) = marcar(zonas, imagen.ve_piso(), 1, ancho, alto);
            match lado_de_bloque(cantidad, RAYOS_PREVIA) {
                1 => (1, if cantidad * 4 <= RAYOS_PREVIA { 2 } else { 1 }, Some(marcados)),
                b => (b, 1, Some(marcar(zonas, imagen.ve_piso(), b, ancho, alto).0)),
            }
        }
    };
    let n = muestras_por_lado as f32;
    let marcados = marcados.as_deref();
    let anterior: &Imagen = imagen;

    let franjas = alto.div_ceil(bloque);
    let siguiente = AtomicUsize::new(0);
    let hilos = std::thread::available_parallelism().map_or(4, |n| n.get());
    let hechas: Vec<(usize, Vec<Pixel>)> = std::thread::scope(|s| {
        let tareas: Vec<_> = (0..hilos)
            .map(|_| {
                s.spawn(|| {
                    let mut hechas = Vec::new();
                    loop {
                        let franja = siguiente.fetch_add(1, Ordering::Relaxed);
                        if franja >= franjas {
                            return hechas;
                        }
                        let desde = franja * bloque;
                        let hasta = (desde + bloque).min(alto);
                        let mut pixeles: Vec<Pixel> = Vec::with_capacity((hasta - desde) * ancho);
                        for y in desde..hasta {
                            for x in 0..ancho {
                                if marcados.is_some_and(|m| !m[y * ancho + x]) {
                                    pixeles.push(anterior.pixel(y * ancho + x));
                                    continue;
                                }
                                if y % bloque != 0 {
                                    pixeles.push(pixeles[pixeles.len() - ancho]);
                                    continue;
                                }
                                if x % bloque != 0 {
                                    pixeles.push(pixeles[pixeles.len() - 1]);
                                    continue;
                                }
                                let mut suma = Capas::cero();
                                let mut piso = false;
                                for sy in 0..muestras_por_lado {
                                    for sx in 0..muestras_por_lado {
                                        let fx = x as f32 + (sx as f32 + 0.5) / n;
                                        let fy = y as f32 + (sy as f32 + 0.5) / n;
                                        let px = (2.0 * (fx / ancho as f32) - 1.0) * aspecto * escala;
                                        let py = (1.0 - 2.0 * (fy / alto as f32)) * escala;
                                        let direccion = (adelante + derecha * px + arriba * py).normalizado();
                                        let (color, al_piso) = trazar(escena, ojo, direccion, PROFUNDIDAD_MAX);
                                        suma = suma + color;
                                        piso |= al_piso;
                                    }
                                }
                                pixeles.push(((suma * (1.0 / (n * n))).a_colores(), piso));
                            }
                        }
                        hechas.push((desde, pixeles));
                    }
                })
            })
            .collect();
        tareas.into_iter().flat_map(|t| t.join().expect("fallo un hilo de render")).collect()
    });

    for (desde, pixeles) in hechas {
        for (i, pixel) in pixeles.into_iter().enumerate() {
            imagen.fijar(desde * ancho + i, pixel);
        }
    }
}

fn guardar_captura(fb: &FrameBuffer, ruta: &str) {
    let (w, h) = fb.dimensions();
    let imagen = image::RgbImage::from_fn(w as u32, h as u32, |x, y| {
        let c = fb.buffer()[y as usize * w + x as usize];
        image::Rgb([(c >> 16) as u8, (c >> 8) as u8, c as u8])
    });
    imagen.save(ruta).unwrap_or_else(|e| panic!("no se pudo guardar {ruta}: {e:?}"));
}

fn valor_argumento(args: &[String], nombre: &str) -> Option<String> {
    args.iter().position(|a| a == nombre).and_then(|i| args.get(i + 1).cloned())
}

fn main() {
    let mut piezas = bus::construir();
    let (decorado, reflectores) = decorado::construir();
    piezas.extend(decorado);
    assert!(piezas.len() <= MAX_GRILLAS, "demasiadas grillas: subir MAX_GRILLAS");

    let fija = |posicion: Vec3, intensidad: f32, sombras: bool| Luz {
        posicion,
        intensidad,
        sombras,
        capa: 0,
        parpadeo: None,
        foco: None,
    };
    let mut luces = vec![
        fija(Vec3::new(5.0, 7.0, 6.0), 1.15, true),
        fija(Vec3::new(-7.0, 2.5, 6.0), 0.5, false),
        fija(Vec3::new(-3.0, 6.0, -8.0), 0.35, false),
    ];
    for r in reflectores {
        assert!(r.capa < CAPAS, "mas reflectores que capas: subir CAPAS");
        luces.push(Luz {
            posicion: r.posicion,
            intensidad: r.intensidad,
            sombras: false,
            capa: r.capa,
            parpadeo: Some(r.parpadeo),
            foco: Some(Foco {
                eje: r.eje,
                cos_centro: 9.0f32.to_radians().cos(),
                cos_borde: 15.0f32.to_radians().cos(),
            }),
        });
    }
    let mut escena = Escena {
        piezas,
        materiales: material::materiales(),
        luces,
        piso: bus::altura_piso(),
    };

    let mut camara = Camara::nueva(bus::centro_escena(), 15.5, AZIMUT_INICIAL, 0.2, 3.0, 25.0);
    let mut framebuffer = FrameBuffer::new(ANCHO, ALTO, Color { r: 0, g: 0, b: 0 });
    let mut imagen = Imagen::nueva(ANCHO, ALTO);

    let args: Vec<String> = std::env::args().collect();
    if let Some(ruta) = valor_argumento(&args, "--captura") {
        let leer = |nombre: &str| valor_argumento(&args, nombre).and_then(|v| v.parse::<f32>().ok());
        if let Some(a) = leer("--azimut") {
            camara.azimut = a;
        }
        if let Some(e) = leer("--elevacion") {
            camara.elevacion = e;
        }
        if let Some(d) = leer("--distancia") {
            camara.distancia = d;
        }
        let tiempo = leer("--tiempo").unwrap_or(0.0);
        escena.animar(tiempo);
        renderizar(&mut imagen, &escena, &camara, Calidad::Final);
        imagen.componer(&mut framebuffer, &escena.encendido(tiempo));
        guardar_captura(&framebuffer, &ruta);
        return;
    }

    let mut window = Window::new(
        "Raytracing - Diorama Bus de Fortnite",
        ANCHO,
        ALTO,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("no se pudo crear la ventana");
    window.set_background_color(0, 0, 0);
    window.set_target_fps(60);

    let musica = Musica::nueva(CANCION, VOLUMEN_INICIAL);
    let mut panel = Panel::nuevo();

    let mut animando = true;
    let mut tiempo = 0.0;
    let mut reloj = Instant::now();
    let mut falta_previa = true;
    let mut falta_final = true;
    let mut ultimo_movimiento = Instant::now();
    let mut mouse_anterior: Option<(f32, f32)> = None;
    let mut zonas_anteriores = Zonas::default();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut cambio = false;

        let paso = reloj.elapsed().as_secs_f32().min(PASO_MAXIMO);
        reloj = Instant::now();
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            animando = !animando;
            falta_final = true;
        }
        if animando {
            tiempo += paso;
            escena.animar(tiempo);
        }

        let (ancho, alto) = window.get_size();
        let tamano = (ancho.max(1), alto.max(1));
        if tamano != framebuffer.dimensions() {
            framebuffer = FrameBuffer::new(tamano.0, tamano.1, Color { r: 0, g: 0, b: 0 });
            imagen = Imagen::nueva(tamano.0, tamano.1);
            cambio = true;
        }

        let controles = [
            (Key::Left, -VELOCIDAD_ROTACION, 0.0, 0.0),
            (Key::Right, VELOCIDAD_ROTACION, 0.0, 0.0),
            (Key::Up, 0.0, VELOCIDAD_ROTACION, 0.0),
            (Key::Down, 0.0, -VELOCIDAD_ROTACION, 0.0),
            (Key::W, 0.0, 0.0, -VELOCIDAD_ZOOM),
            (Key::S, 0.0, 0.0, VELOCIDAD_ZOOM),
        ];
        for (tecla, azimut, elevacion, zoom) in controles {
            if window.is_key_down(tecla) {
                camara.rotar(azimut, elevacion);
                camara.zoom(zoom);
                cambio = true;
            }
        }

        let presionado = window.get_mouse_down(MouseButton::Left);
        let mouse = window.get_mouse_pos(MouseMode::Pass);
        let rueda = window.get_scroll_wheel().map(|(_, y)| y);
        let (ancho_fb, alto_fb) = framebuffer.dimensions();
        let en_panel = match &musica {
            Some(musica) => panel.atender(musica, mouse, presionado, rueda, ancho_fb, alto_fb),
            None => false,
        };
        let arrastrando = presionado && !en_panel;
        if arrastrando {
            if let (Some((x0, y0)), Some((x, y))) = (mouse_anterior, mouse) {
                let (dx, dy) = (x - x0, y - y0);
                if dx != 0.0 || dy != 0.0 {
                    camara.rotar(-dx * SENSIBILIDAD_MOUSE, dy * SENSIBILIDAD_MOUSE);
                    cambio = true;
                }
            }
            mouse_anterior = mouse;
        } else {
            mouse_anterior = None;
        }
        if let (false, Some(rueda)) = (en_panel, rueda) {
            camara.zoom(-rueda * VELOCIDAD_ZOOM_RUEDA);
            cambio = true;
        }

        if cambio {
            falta_previa = true;
            falta_final = true;
            ultimo_movimiento = Instant::now();
        }
        let quieta = !arrastrando && ultimo_movimiento.elapsed() >= ESPERA_FINAL;
        let (ancho, alto) = imagen.dimensiones();
        if falta_previa || (falta_final && animando && !quieta) {
            renderizar(&mut imagen, &escena, &camara, Calidad::Previa);
            falta_previa = false;
        } else if falta_final && quieta {
            renderizar(&mut imagen, &escena, &camara, Calidad::Final);
            falta_final = false;
            zonas_anteriores = zonas_en_movimiento(&escena, &camara, ancho, alto);
        } else if animando {
            let zonas = zonas_en_movimiento(&escena, &camara, ancho, alto);
            let a_renderizar = zonas.unir(&zonas_anteriores);
            renderizar(&mut imagen, &escena, &camara, Calidad::Zonas(&a_renderizar));
            zonas_anteriores = zonas;
        }
        imagen.componer(&mut framebuffer, &escena.encendido(tiempo));
        if let Some(musica) = &musica {
            musica.actualizar();
            panel.dibujar(&mut framebuffer, musica, mouse);
        }

        let (w, h) = framebuffer.dimensions();
        window
            .update_with_buffer(framebuffer.buffer(), w, h)
            .expect("no se pudo actualizar la ventana");
    }
}
