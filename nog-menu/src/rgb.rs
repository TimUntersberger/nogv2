#[derive(Clone, Copy, Debug, Default)]
pub struct Rgb(pub [f32; 3]);

impl Rgb {
    pub const WHITE: Rgb = Rgb([1.0, 1.0, 1.0]);
    pub const BLACK: Rgb = Rgb([0.0, 0.0, 0.0]);

    pub fn from_hex(hex: i32) -> Self {
        Rgb([
            ((hex >> 16) & 0xFF) as f32 / 255.0,
            ((hex >> 8) & 0xFF) as f32 / 255.0,
            (hex & 0xFF) as f32 / 255.0,
        ])
    }

    pub fn to_hex(self) -> i32 {
        (((self.0[0] * 255.0) as i32 & 0xff) << 16)
            + (((self.0[1] * 255.0) as i32 & 0xff) << 8)
            + ((self.0[2] * 255.0) as i32 & 0xff)
    }

    pub fn scaled(&self, factor: f32) -> Self {
        let [mut red, mut green, mut blue] = self.0;

        red *= factor;
        green *= factor;
        blue *= factor;

        Self([red.min(1.0), green.min(1.0), blue.min(1.0)])
    }
}
