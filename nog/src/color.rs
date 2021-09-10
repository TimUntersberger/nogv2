pub type RGB = [f32; 3];

pub fn rgb_to_hex(rgb: RGB) -> i32 {
    (((rgb[0] * 255.0) as i32 & 0xff) << 16)
        + (((rgb[1] * 255.0) as i32 & 0xff) << 8)
        + ((rgb[2] * 255.0) as i32 & 0xff)
}

pub fn hex_to_rgb(hex: i32) -> RGB {
    [
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    ]
}

pub fn scale_color(color: i32, factor: f64) -> i32 {
    let [mut red, mut green, mut blue] = hex_to_rgb(color);

    red = (red as f64 * factor).round() as f32;
    green = (green as f64 * factor).round() as f32;
    blue = (blue as f64 * factor).round() as f32;

    rgb_to_hex([red, green, blue])
}
