#[derive(Clone, Copy, Debug)]
pub struct RGB(pub [f32; 3]);

impl RGB {
    pub const WHITE: RGB = RGB([255.0, 255.0, 255.0]);
    pub const BLACK: RGB = RGB([0.0, 0.0, 0.0]);

    pub fn from_hex(hex: i32) -> Self {
        RGB([
            ((hex >> 16) & 0xFF) as f32 / 255.0,
            ((hex >> 8) & 0xFF) as f32 / 255.0,
            (hex & 0xFF) as f32 / 255.0,
        ])
    }

    pub fn to_hex(&self) -> i32 {
        (((self.0[0] * 255.0) as i32 & 0xff) << 16)
            + (((self.0[1] * 255.0) as i32 & 0xff) << 8)
            + ((self.0[2] * 255.0) as i32 & 0xff)
    }

    pub fn scaled(&self, factor: f32) -> Self {
        let [mut red, mut green, mut blue] = self.0;

        red = (red * factor).round();
        green = (green * factor).round();
        blue = (blue * factor).round();

        Self([red, green, blue])
    }
}
