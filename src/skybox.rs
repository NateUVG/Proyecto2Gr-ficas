use crate::vector::Vec3;

fn azar(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ (z as u32).wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0xffff) as f32 / 65_535.0
}

fn suave(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn transicion(borde0: f32, borde1: f32, x: f32) -> f32 {
    suave(((x - borde0) / (borde1 - borde0)).clamp(0.0, 1.0))
}

pub fn mezclar(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a * (1.0 - t) + b * t
}

fn ruido(p: Vec3) -> f32 {
    let (ix, iy, iz) = (p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
    let (fx, fy, fz) = (suave(p.x - ix as f32), suave(p.y - iy as f32), suave(p.z - iz as f32));
    let valor = |dx: i32, dy: i32, dz: i32| azar(ix + dx, iy + dy, iz + dz);
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x00 = lerp(valor(0, 0, 0), valor(1, 0, 0), fx);
    let x10 = lerp(valor(0, 1, 0), valor(1, 1, 0), fx);
    let x01 = lerp(valor(0, 0, 1), valor(1, 0, 1), fx);
    let x11 = lerp(valor(0, 1, 1), valor(1, 1, 1), fx);
    lerp(lerp(x00, x10, fy), lerp(x01, x11, fy), fz)
}

fn ruido_fractal(p: Vec3) -> f32 {
    0.6 * ruido(p) + 0.3 * ruido(p * 2.03 + Vec3::new(5.2, 1.3, 7.7)) + 0.1 * ruido(p * 4.1 + Vec3::new(2.8, 9.1, 3.4))
}

const MANCHAS_DE_LUZ: [(Vec3, f32); 3] = [
    (Vec3 { x: -0.45, y: 0.15, z: -1.0 }, 0.5),
    (Vec3 { x: -0.58, y: 0.2, z: 0.81 }, 0.55),
    (Vec3 { x: 0.99, y: 0.3, z: 0.11 }, 0.5),
];

pub fn color_lobby(d: Vec3) -> Vec3 {
    let horizonte = Vec3::new(0.35, 0.35, 0.80);
    let techo = Vec3::new(0.17, 0.10, 0.45);
    let mut color = mezclar(horizonte, techo, transicion(-0.02, 0.35, d.y));

    let hacia_resplandor = MANCHAS_DE_LUZ[0].0.normalizado();
    let resplandor = d.dot(hacia_resplandor).max(0.0).powf(4.0);
    color = color + Vec3::new(0.12, 0.10, 0.22) * resplandor;
    color = color + Vec3::new(0.08, 0.06, 0.12) * (ruido(d * 2.2) - 0.5);

    let mut luz: f32 = 0.0;
    for (direccion, apertura) in MANCHAS_DE_LUZ {
        luz = luz.max(transicion(apertura.cos(), 1.0, d.dot(direccion.normalizado())));
    }
    let luz = (luz * (0.75 + 0.5 * ruido_fractal(d * 1.6))).min(1.0);
    color * (0.4 + 0.6 * luz)
}

pub fn color_piso(p: Vec3) -> Vec3 {
    let base = Vec3::new(0.21, 0.23, 0.63);
    base + Vec3::new(0.03, 0.03, 0.06) * (ruido(p * 0.8) - 0.5)
}
