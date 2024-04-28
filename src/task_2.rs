use image::{GenericImage, GenericImageView, Pixel};
use std::path::PathBuf;

use crate::im;

pub fn run(target: &PathBuf, width_downsize: u32) {
    if !target.exists() {
        eprintln!("path was not found `{}`", target.to_string_lossy());
        return;
    }
    // @check ext
    if target.is_file() {
        resize_image(target, width_downsize);
    }
    if target.is_dir() {
        let read_dir = std::fs::read_dir(target).expect("read dir");
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().unwrap_or_default().to_str().expect("utf8");
                if matches!(ext, "png" | "jpg" | "jpeg") {
                    resize_image(&path, width_downsize);
                }
            }
        }
    }
}

fn resize_image(path: &PathBuf, mut width_downsize: u32) {
    let name = path
        .file_stem()
        .expect("image filename")
        .to_str()
        .expect("utf8 filename");

    let results_path = PathBuf::from("image_process_results");
    if !results_path.exists() {
        std::fs::create_dir(&results_path).expect("dir created");
    }
    let image_dir = results_path.join(name);
    if !image_dir.exists() {
        std::fs::create_dir(&image_dir).expect("dir created");
    }

    let mut image = im::image_open(&path);

    // limiting downsize amount
    if image.width() <= width_downsize {
        width_downsize = image.width() - 1;
    }

    // visualize scaled up gradient image
    let gradient = gradient_magnitude(&image, 100);
    im::image_buffer_luma16_save_png(gradient, &image_dir.join(format!("{name}_gradient.png")));

    let mut visualize = true;
    for _ in 0..width_downsize {
        let grad = gradient_magnitude(&image, 1);
        let table = DPTable::from_gradient_buffer(&grad);
        let path = Path::from_dp_table(&table);

        if visualize {
            visualize = false;

            let mut image_copy = image.clone();
            visualize_path(&mut image_copy, &path);

            im::image_buffer_save_png(
                image_copy.into_rgb8(),
                &image_dir.join(format!("{name}_removed_path.png")),
            );

            im::image_buffer_luma16_save_png(
                table.clone().to_gradient_buffer(),
                &image_dir.join(format!("{name}_dp_table_weights.png")),
            );
        }
        remove_path(&mut image, path);
    }

    im::image_buffer_save_png(
        image.into_rgb8(),
        &image_dir.join(format!("{name}_resized.png")),
    );
}

type GradientBuffer = image::ImageBuffer<image::Luma<u16>, Vec<u16>>;

fn gradient_magnitude(image: &image::DynamicImage, visual_scale: u16) -> GradientBuffer {
    let (red, green, blue) = decompose_channels(image);
    let r_grad = imageproc::gradients::sobel_gradients(red.as_luma8().unwrap());
    let g_grad = imageproc::gradients::sobel_gradients(green.as_luma8().unwrap());
    let b_grad = imageproc::gradients::sobel_gradients(blue.as_luma8().unwrap());

    let (w, h) = r_grad.dimensions();
    let mut container = Vec::with_capacity((w * h) as usize);
    for (r, g, b) in itertools::izip!(r_grad.pixels(), g_grad.pixels(), b_grad.pixels()) {
        container.push(r[0] * visual_scale + g[0] * visual_scale + b[0] * visual_scale);
    }

    image::ImageBuffer::from_raw(w, h, container).unwrap()
}

fn decompose_channels(
    image: &image::DynamicImage,
) -> (
    image::DynamicImage,
    image::DynamicImage,
    image::DynamicImage,
) {
    let w = image.width();
    let h = image.height();
    let mut red = image::DynamicImage::new_luma8(w, h);
    let mut green = image::DynamicImage::new_luma8(w, h);
    let mut blue = image::DynamicImage::new_luma8(w, h);

    for (x, y, pixel) in image.pixels() {
        let r = pixel[0];
        let g = pixel[1];
        let b = pixel[2];
        red.put_pixel(x, y, *image::Rgba::from_slice(&[r, r, r, 255]));
        green.put_pixel(x, y, *image::Rgba::from_slice(&[g, g, g, 255]));
        blue.put_pixel(x, y, *image::Rgba::from_slice(&[b, b, b, 255]));
    }

    (red, green, blue)
}

fn remove_path(image: &mut image::DynamicImage, path: Path) {
    let image_buffer = image.to_rgb8();
    let (w, h) = image_buffer.dimensions();
    let container = image_buffer.into_raw();
    let mut new_pixels = vec![];

    let mut path = path.indices.iter();
    let mut i = 0;
    while let Some(&index) = path.next() {
        new_pixels.extend(&container[i..index * 3]);
        i = (index + 1) * 3;
    }

    new_pixels.extend(&container[i..]);
    let ib = image::ImageBuffer::from_raw(w - 1, h, new_pixels)
        .expect("remove_path: failed to create image_buffer");
    *image = image::DynamicImage::ImageRgb8(ib);
}

fn visualize_path(image: &mut image::DynamicImage, path: &Path) {
    for (x, y) in path.coords_x_y.iter().cloned() {
        image.put_pixel(x, y, image::Rgba::<u8>([253, 218, 13, 255]));
    }
}

#[derive(Clone)]
struct DPTable {
    width: usize,
    height: usize,
    table: Vec<u16>,
}

// TODO: horizontal
impl DPTable {
    fn get(&self, w: usize, h: usize) -> u16 {
        let i = self.width * h + w;
        self.table[i]
    }

    fn set(&mut self, w: usize, h: usize, v: u16) {
        let i = self.width * h + w;
        self.table[i] = v;
    }

    fn to_gradient_buffer(self) -> GradientBuffer {
        GradientBuffer::from_raw(self.width as u32, self.height as u32, self.table).unwrap()
    }

    fn path_start_index(&self) -> usize {
        self.table
            .iter()
            .take(self.width)
            .enumerate()
            .map(|(i, n)| (n, i))
            .min()
            .map(|(_, i)| i)
            .unwrap()
    }

    fn from_gradient_buffer(gradient: &GradientBuffer) -> DPTable {
        let dims = gradient.dimensions();
        let w = dims.0 as usize;
        let h = dims.1 as usize;
        let mut table = DPTable {
            width: w,
            height: h,
            table: vec![0; w * h],
        };
        // return gradient[h][w]
        let get = |w, h| gradient.get_pixel(w as u32, h as u32)[0];

        // Initialize bottom row
        for i in 0..w {
            let px = get(i, h - 1);
            table.set(i, h - 1, px)
        }
        // For each cell in row j, select the smaller of the cells in the
        // row above. Special case the end rows
        for row in (0..h - 1).rev() {
            for col in 1..w - 1 {
                let l = table.get(col - 1, row + 1);
                let m = table.get(col, row + 1);
                let r = table.get(col + 1, row + 1);
                table.set(col, row, get(col, row) + l.min(m).min(r));
            }
            // special case far left and far right:
            let left = get(0, row) + (table.get(0, row + 1).min(table.get(1, row + 1)));
            table.set(0, row, left);
            let right = get(0, row) + (table.get(w - 1, row + 1).min(table.get(w - 2, row + 1)));
            table.set(w - 1, row, right);
        }
        table
    }
}

struct Path {
    indices: Vec<usize>,
    coords_x_y: Vec<(u32, u32)>,
}

impl Path {
    fn from_dp_table(table: &DPTable) -> Path {
        let mut v = Vec::with_capacity(table.height);
        let mut coords_x_y = Vec::with_capacity(table.height);

        let mut col: usize = table.path_start_index();
        v.push(col);

        for row in 1..table.height {
            if col == 0 {
                let m = table.get(col, row);
                let r = table.get(col + 1, row);
                if m > r {
                    col += 1;
                }
            } else if col == table.width - 1 {
                let l = table.get(col - 1, row);
                let m = table.get(col, row);
                if l < m {
                    col -= 1;
                }
            } else {
                let l = table.get(col - 1, row);
                let m = table.get(col, row);
                let r = table.get(col + 1, row);
                let minimum = l.min(m).min(r);
                if minimum == l {
                    col -= 1;
                } else if minimum == r {
                    col += 1;
                }
            }
            v.push(col + row * table.width);
            coords_x_y.push((col as u32, row as u32));
        }

        Path {
            indices: v,
            coords_x_y,
        }
    }
}
