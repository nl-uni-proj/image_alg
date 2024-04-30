use std::{ffi::OsStr, path::PathBuf};

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

pub struct ImageInfo {
    image_dir: PathBuf,
    name: String,
}

impl ImageInfo {
    pub fn save_path_concat(&self, postfix: &str, format: image::ImageFormat) -> PathBuf {
        self.image_dir.join(format!(
            "{}_{}.{}",
            self.name,
            postfix,
            format.extensions_str()[0]
        ))
    }
}

pub fn open_and_setup_output(target: &PathBuf) -> Vec<(image::DynamicImage, ImageInfo)> {
    if !target.exists() {
        eprintln!("path was not found `{}`", target.to_string_lossy());
        return Vec::new();
    }

    let mut image_paths = Vec::new();
    if target.is_file() {
        if ext_is_supported(target.extension()) {
            image_paths.push(target.clone());
        }
    }
    if target.is_dir() {
        let read_dir = std::fs::read_dir(target).expect("read dir");
        for entry in read_dir.flatten() {
            let path = entry.path();
            if ext_is_supported(path.extension()) {
                image_paths.push(path);
            }
        }
    }

    let mut image_inputs = Vec::new();

    let results_path = PathBuf::from("image_process_results");
    if !results_path.exists() {
        std::fs::create_dir(&results_path).expect("dir created");
    }

    for path in image_paths {
        let name = path
            .file_stem()
            .expect("image filename")
            .to_str()
            .expect("utf8 filename");
        let image_dir = results_path.join(name);
        if !image_dir.exists() {
            std::fs::create_dir(&image_dir).expect("dir created");
        }
        image_inputs.push((
            image_open(&path),
            ImageInfo {
                image_dir,
                name: name.to_string(),
            },
        ));
    }

    image_inputs
}

fn ext_is_supported(ext: Option<&OsStr>) -> bool {
    let ext = ext.unwrap_or_default().to_str().expect("utf8");
    matches!(ext, "png" | "jpg" | "jpeg")
}

fn image_open(path: &PathBuf) -> image::DynamicImage {
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

pub fn image_buffer_luma16_save_png(
    buffer: image::ImageBuffer<image::Luma<u16>, Vec<u16>>,
    save_path: &PathBuf,
) {
    buffer
        .save_with_format(save_path, image::ImageFormat::Png)
        .expect("save with format");
    println!("saved: `{}`", save_path.to_string_lossy());
}
