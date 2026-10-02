use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::fs::File;
use std::time::Duration;

pub struct Musica {
    _salida: MixerDeviceSink,
    reproductor: Player,
    ruta: String,
    duracion: Option<Duration>,
}

impl Musica {
    pub fn nueva(ruta: &str, volumen: f32) -> Option<Self> {
        let mut salida = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| eprintln!("sin audio: {e}"))
            .ok()?;
        salida.log_on_drop(false);
        let reproductor = Player::connect_new(salida.mixer());
        reproductor.set_volume(volumen);
        let mut musica = Musica { _salida: salida, reproductor, ruta: ruta.to_string(), duracion: None };
        musica.duracion = musica.cargar()?;
        Some(musica)
    }

    fn cargar(&self) -> Option<Option<Duration>> {
        let archivo = File::open(&self.ruta).map_err(|e| eprintln!("no se pudo abrir {}: {e}", self.ruta)).ok()?;
        let cancion = Decoder::try_from(archivo).map_err(|e| eprintln!("no se pudo leer {}: {e}", self.ruta)).ok()?;
        let duracion = cancion.total_duration();
        self.reproductor.append(cancion);
        Some(duracion)
    }

    pub fn actualizar(&self) {
        if self.reproductor.empty() {
            self.cargar();
        }
    }

    pub fn pausada(&self) -> bool {
        self.reproductor.is_paused()
    }

    pub fn alternar_pausa(&self) {
        if self.pausada() {
            self.reproductor.play();
        } else {
            self.reproductor.pause();
        }
    }

    pub fn volumen(&self) -> f32 {
        self.reproductor.volume()
    }

    pub fn fijar_volumen(&self, volumen: f32) {
        self.reproductor.set_volume(volumen.clamp(0.0, 1.0));
    }

    pub fn posicion(&self) -> Duration {
        self.reproductor.get_pos()
    }

    pub fn duracion(&self) -> Option<Duration> {
        self.duracion
    }

    pub fn ir_a(&self, posicion: Duration) {
        let posicion = match self.duracion {
            Some(total) => posicion.min(total.saturating_sub(Duration::from_millis(50))),
            None => posicion,
        };
        if let Err(e) = self.reproductor.try_seek(posicion) {
            eprintln!("no se pudo mover la cancion: {e}");
        }
    }

    pub fn adelantar(&self, segundos: f32) {
        self.ir_a(self.posicion() + Duration::from_secs_f32(segundos));
    }

    pub fn reiniciar(&self) {
        self.ir_a(Duration::ZERO);
    }
}
