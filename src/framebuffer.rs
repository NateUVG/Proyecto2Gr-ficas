#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn as_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
    }
}

pub struct FrameBuffer {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    background_color: Color,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize, background_color: Color) -> Self {
        FrameBuffer {
            width,
            height,
            buffer: vec![background_color.as_u32(); width * height],
            background_color,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.fill(self.background_color.as_u32());
    }

    pub fn point(&mut self, x: usize, y: usize, color: Color) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = color.as_u32();
        }
    }

    pub fn point_translucido(&mut self, x: usize, y: usize, color: Color, alfa: f32) {
        if x < self.width && y < self.height {
            let i = y * self.width + x;
            let anterior = self.buffer[i];
            let mezclar = |nuevo: u8, desplazamiento: u32| {
                let viejo = ((anterior >> desplazamiento) & 0xff) as f32;
                (viejo + (nuevo as f32 - viejo) * alfa) as u32
            };
            self.buffer[i] = (mezclar(color.r, 16) << 16) | (mezclar(color.g, 8) << 8) | mezclar(color.b, 0);
        }
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}
