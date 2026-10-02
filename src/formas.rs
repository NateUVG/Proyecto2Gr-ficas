use crate::vector::Vec3;

pub fn caja_redondeada(p: Vec3, min: Vec3, max: Vec3, radio: f32) -> f32 {
    let centro = (min + max) * 0.5;
    let medio = (max - min) * 0.5;
    let q = Vec3::new(
        (p.x - centro.x).abs() - medio.x + radio,
        (p.y - centro.y).abs() - medio.y + radio,
        (p.z - centro.z).abs() - medio.z + radio,
    );
    let afuera = Vec3::new(q.x.max(0.0), q.y.max(0.0), q.z.max(0.0)).length();
    let adentro = q.x.max(q.y).max(q.z).min(0.0);
    afuera + adentro - radio
}

pub fn cilindro(p: Vec3, centro: Vec3, eje: usize, radio: f32, largo: f32, redondeo: f32) -> f32 {
    let d = p - centro;
    let a = d.eje(eje);
    let radial = (d.dot(d) - a * a).max(0.0).sqrt();
    let (qr, qa) = (radial - radio + redondeo, a.abs() - largo * 0.5 + redondeo);
    let afuera = (qr.max(0.0).powi(2) + qa.max(0.0).powi(2)).sqrt();
    afuera + qr.max(qa).min(0.0) - redondeo
}

pub fn capsula(p: Vec3, a: Vec3, b: Vec3, radio: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
    (pa - ba * h).length() - radio
}

pub fn cono_redondeado(p: Vec3, a: Vec3, b: Vec3, ra: f32, rb: f32) -> f32 {
    let ba = b - a;
    let l2 = ba.dot(ba);
    let rr = ra - rb;
    let a2 = l2 - rr * rr;
    let il2 = 1.0 / l2;

    let pa = p - a;
    let y = pa.dot(ba);
    let z = y - l2;
    let lateral = pa * l2 - ba * y;
    let x2 = lateral.dot(lateral);
    let y2 = y * y * l2;
    let z2 = z * z * l2;

    let k = rr.signum() * rr * rr * x2;
    if z.signum() * a2 * z2 > k {
        return (x2 + z2).sqrt() * il2 - rb;
    }
    if y.signum() * a2 * y2 < k {
        return (x2 + y2).sqrt() * il2 - ra;
    }
    ((x2 * a2 * il2).sqrt() + y * rr) * il2 - ra
}

pub fn toro(p: Vec3, centro: Vec3, eje: usize, radio_mayor: f32, radio_menor: f32) -> f32 {
    let d = p - centro;
    let a = d.eje(eje);
    let radial = (d.dot(d) - a * a).max(0.0).sqrt();
    ((radial - radio_mayor).powi(2) + a * a).sqrt() - radio_menor
}

pub fn barra(p: Vec3, a: Vec3, b: Vec3, radio: f32, redondeo: f32) -> f32 {
    let largo = (b - a).length();
    let eje = (b - a) * (1.0 / largo);
    let pa = p - a;
    let axial = pa.dot(eje);
    let radial = (pa - eje * axial).length();
    let (qr, qa) = (radial - radio + redondeo, (axial - largo * 0.5).abs() - largo * 0.5 + redondeo);
    let afuera = (qr.max(0.0).powi(2) + qa.max(0.0).powi(2)).sqrt();
    afuera + qr.max(qa).min(0.0) - redondeo
}
