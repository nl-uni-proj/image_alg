use std::path::PathBuf;

pub const COLOR_BLACK: image::Rgb<u8> = image::Rgb([0, 0, 0]);
pub const COLOR_WHITE: image::Rgb<u8> = image::Rgb([255, 255, 255]);
pub const COLOR_RED: image::Rgb<u8> = image::Rgb([230, 50, 50]);
pub const COLOR_GREEN: image::Rgb<u8> = image::Rgb([34, 139, 34]);

#[derive(Copy, Clone)]
pub struct RgbF {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl RgbF {
    pub fn new(r: f32, g: f32, b: f32) -> RgbF {
        RgbF { r, g, b }
    }

    pub fn from_u8(color: image::Rgb<u8>) -> RgbF {
        RgbF {
            r: color[0] as f32 / 255.0,
            g: color[1] as f32 / 255.0,
            b: color[2] as f32 / 255.0,
        }
    }

    pub fn into_u8(&self) -> image::Rgb<u8> {
        image::Rgb([
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        ])
    }
}

pub fn image_open(path: &PathBuf) -> image::DynamicImage {
    let image = image::open(path).expect("image open");
    println!(
        "{}",
        format!(
            "opened: `{}`, color: `{:?}`, size `{}x{}`",
            path.to_string_lossy(),
            image.color(),
            image.width(),
            image.height()
        )
    );
    image
}

pub fn image_buffer_save_png(
    buffer: image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    save_path: &PathBuf,
) {
    buffer
        .save_with_format(save_path, image::ImageFormat::Png)
        .expect("save with format");
    println!("saved: `{}`", save_path.to_string_lossy());
}
